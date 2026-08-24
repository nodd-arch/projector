<script>
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { onMount, onDestroy } from 'svelte';
  import ScriptureDisplay from '$lib/ScriptureDisplay.svelte';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import {
    translationId,
    projectedRef,
    isPanicked,
    livePreview,
    mode,
    currentSessionId,
  } from '$lib/stores/session.js';

  let query = '';
  let translations = [];
  let results = [];
  let error = null;
  let searchInput;
  let projectionStatus = '';

  let bookFilter = '';

  $: otBooks = books.filter((b) => b.testamentid === 1 && b.name.toLowerCase().includes(bookFilter.toLowerCase()));
  $: ntBooks = books.filter((b) => b.testamentid === 2 && b.name.toLowerCase().includes(bookFilter.toLowerCase()));

  let unlistenPreview;

  let displaySettings = {
    fontFamily: 'Poppins',
    weightNormal: '400',
    weightWj: '700',
    colorNormal: '#ffffff',
    colorWj: '#e63946',
    colorBracket: '#ffffff',
  };

  let background = { kind: 'color', value: '#000000', opacity: 1 };

  function loadDisplaySettings() {
    displaySettings = {
      fontFamily: localStorage.getItem('displayFontFamily') || displaySettings.fontFamily,
      weightNormal: localStorage.getItem('displayWeightNormal') || displaySettings.weightNormal,
      weightWj: localStorage.getItem('displayWeightWj') || displaySettings.weightWj,
      colorNormal: localStorage.getItem('displayColorNormal') || displaySettings.colorNormal,
      colorWj: localStorage.getItem('displayColorWj') || displaySettings.colorWj,
      colorBracket: localStorage.getItem('displayColorBracket') || displaySettings.colorBracket,
    };
  }

  // Convert local Windows paths into a webview-loadable URL. http(s) URLs pass through untouched.
  $: resolvedBgValue =
    (background.kind === 'image' || background.kind === 'video') &&
    background.value &&
    !background.value.startsWith('http')
      ? convertFileSrc(background.value)
      : background.value;

  $: bgStyle = background.kind === 'image'
    ? `background-image: url('${resolvedBgValue}'); background-size: cover; background-position: ${background.position || 'left'} center;`
    : background.kind === 'gradient'
    ? `background: ${background.value};`
    : `background-color: ${background.value};`;

  // Browse state
  let books = [];
  let selectedBook = null;
  let chapterCount = 0;
  let selectedChapter = null;
  let chapterVerses = [];

  // History state
  let historyEntries = [];

  async function switchMode(newMode) {
    $mode = newMode;
    error = null;
    if (newMode === 'browse' && books.length === 0) {
      try {
        books = await invoke('get_books');
      } catch (e) {
        error = e;
      }
    }
    if (newMode === 'history') {
      try {
        historyEntries = await invoke('get_history');
      } catch (e) {
        error = e;
      }
    }
  }

  async function pickBook(book) {
    selectedBook = book;
    selectedChapter = null;
    chapterVerses = [];
    try {
      chapterCount = await invoke('get_chapter_count', { bookid: book.bookid, translationId: $translationId });
    } catch (e) {
      error = e;
    }
  }

  async function pickChapter(chapterNum) {
    selectedChapter = chapterNum;
    try {
      chapterVerses = await invoke('get_chapter_verses', {
        bookid: selectedBook.bookid,
        chapternumber: chapterNum,
        translationId: $translationId,
      });
    } catch (e) {
      error = e;
    }
  }

  async function projectFromHistory(entry) {
    $translationId = entry.verse.translationid;
    await project(entry.verse);
  }

  onMount(async () => {
    loadDisplaySettings();
    await listen('display-settings-update', (event) => {
      displaySettings = {
        fontFamily: event.payload.font_family,
        weightNormal: event.payload.weight_normal,
        weightWj: event.payload.weight_wj,
        colorNormal: event.payload.color_normal,
        colorWj: event.payload.color_wj,
        colorBracket: event.payload.color_bracket,
      };
    });

    const storedBg = localStorage.getItem('background');
    if (storedBg) background = JSON.parse(storedBg);
    await listen('background-update', (e) => {
      background = e.payload;
      localStorage.setItem('background', JSON.stringify(e.payload));
    });

    try {
      translations = await invoke('get_translations');
      if (translations.length > 0) $translationId = translations[0].translationid;
    } catch (e) {
      error = e;
    }

    unlistenPreview = await listen('preview-sync', (event) => {
      $livePreview = event.payload;
      $isPanicked = false;
    });

    window.addEventListener('keydown', handleGlobalKeydown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleGlobalKeydown);
    unlistenPreview?.();
  });

  function isTypingContext() {
    const tag = document.activeElement?.tagName;
    return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT';
  }

  function computeLive(v, abbr) {
    return !!(
      $projectedRef &&
      $livePreview &&
      $projectedRef.bookid === v.bookid &&
      $projectedRef.chapternumber === v.chapternumber &&
      $projectedRef.versenumber === v.versenumber &&
      $livePreview.translation_abbr === abbr
    );
  }

  $: currentAbbr = translations.find((t) => t.translationid === $translationId)?.abbreviation;

  $: resultsWithLive = results.map((v) => ({ verse: v, live: computeLive(v, currentAbbr) }));
  $: chapterVersesWithLive = chapterVerses.map((v) => ({ verse: v, live: computeLive(v, currentAbbr) }));

  // Group the flat history log into branching trails by session_id.
  // Each deliberate Project click starts a new session; arrow-key stepping
  // continues the current one. Newest trail shown first; each trail stays
  // chronological internally so the "visited verses" strip reads left-to-right.
  $: sessions = (() => {
    const map = new Map();
    for (const entry of historyEntries) {
      if (!map.has(entry.session_id)) map.set(entry.session_id, []);
      map.get(entry.session_id).push(entry);
    }
    return Array.from(map.values()).reverse();
  })();

  async function handleGlobalKeydown(e) {
    error = null;

    if (e.ctrlKey && e.key.toLowerCase() === 'f') {
      e.preventDefault();
      searchInput?.focus();
      searchInput?.select();
      return;
    }

    if (e.key === 'Escape') {
      try {
        if ($isPanicked) {
          await invoke('panic_restore');
          $isPanicked = false;
        } else {
          await invoke('panic_clear');
          $isPanicked = true;
        }
      } catch (err) {
        error = err;
      }
      return;
    }

    if (isTypingContext()) return;

    if (e.key >= '1' && e.key <= '9') {
      const idx = parseInt(e.key, 10) - 1;
      if (translations[idx]) await handleTranslationChange(translations[idx].translationid);
      return;
    }

    if (e.key === 'Tab') {
      e.preventDefault();
      const currentIdx = translations.findIndex((t) => t.translationid === $translationId);
      const nextIdx = (currentIdx + 1) % translations.length;
      await handleTranslationChange(translations[nextIdx].translationid);
      return;
    }

    if (e.key === 'ArrowRight' || e.key === 'ArrowLeft') {
      if (!$projectedRef) return;
      const direction = e.key === 'ArrowRight' ? 1 : -1;
      const t = translations.find((t) => t.translationid === $translationId);
      try {
        const verse = await invoke('navigate_projection', {
          bookid: $projectedRef.bookid,
          chapternumber: $projectedRef.chapternumber,
          versenumber: $projectedRef.versenumber,
          translationId: $translationId,
          translationAbbr: t ? t.abbreviation : '',
          direction,
        });
        if (verse) {
          $projectedRef = {
            bookid: verse.bookid,
            chapternumber: verse.chapternumber,
            versenumber: verse.versenumber,
          };
          try {
            await invoke('add_history_entry', {
              verse,
              translationAbbr: t ? t.abbreviation : '',
              sessionId: $currentSessionId,
            });
          } catch (e) { /* non-critical, don't surface as an error */ }
        }
      } catch (err) {
        error = err;
      }
      return;
    }
  }

  async function removeProjection() {
    if (!$livePreview || $isPanicked) return;
    error = null;
    try {
      await invoke('panic_clear');
      $isPanicked = true;
    } catch (e) {
      error = e;
    }
  }

  async function restoreProjection() {
    if (!$isPanicked) return;
    error = null;
    try {
      await invoke('panic_restore');
      $isPanicked = false;
    } catch (e) {
      error = e;
    }
  }

  async function handleTranslationChange(newTranslationId) {
    $translationId = newTranslationId;
    if ($projectedRef) {
      const t = translations.find((t) => t.translationid === newTranslationId);
      try {
        await invoke('switch_projection_translation', {
          bookid: $projectedRef.bookid,
          chapternumber: $projectedRef.chapternumber,
          versenumber: $projectedRef.versenumber,
          translationId: newTranslationId,
          translationAbbr: t ? t.abbreviation : '',
        });
      } catch (e) {
        error = e;
      }
    }
    if (query) await runSearch();
  }

  async function runSearch() {
    error = null;
    try {
      results = await invoke('search_scripture', { query, translationId: $translationId });
    } catch (e) {
      error = e;
      results = [];
    }
  }

  async function project(verse) {
    if (computeLive(verse, currentAbbr)) return;
    error = null;
    $isPanicked = false;
    const t = translations.find((t) => t.translationid === $translationId);
    $projectedRef = {
      bookid: verse.bookid,
      chapternumber: verse.chapternumber,
      versenumber: verse.versenumber,
    };
    $currentSessionId = Date.now(); // deliberate project = new branch/session
    try {
      await invoke('push_to_projection', {
        verse,
        translationAbbr: t ? t.abbreviation : '',
      });
      await invoke('add_history_entry', {
        verse,
        translationAbbr: t ? t.abbreviation : '',
        sessionId: $currentSessionId,
      });
    } catch (e) {
      error = e;
    }
  }

  let projectionOpen = false;

  async function toggleProjection() {
    error = null;
    try {
      if (projectionOpen) {
        await invoke('hide_projection_window');
        projectionOpen = false;
        projectionStatus = 'Projection window hidden';
      } else {
        const width = parseInt(localStorage.getItem('outputWidth')) || 1920;
        const height = parseInt(localStorage.getItem('outputHeight')) || 1080;
        projectionStatus = await invoke('locate_and_project', { width, height });
        projectionOpen = true;
      }
    } catch (e) {
      error = e;
    }
  }
</script>

<div class="console">
  <div class="workspace">
    <div class="toolbar">
      <button on:click={toggleProjection}>
        {projectionOpen ? 'Hide Projection Window' : 'Open Projection Window'}
      </button>
      {#if projectionStatus}<span class="status">{projectionStatus}</span>{/if}
      <a class="settings-link" href="/settings">Settings</a>
    </div>

    <div class="mode-tabs">
      <button class:active={$mode === 'search'} on:click={() => switchMode('search')}>Search</button>
      <button class:active={$mode === 'browse'} on:click={() => switchMode('browse')}>Browse</button>
      <button class:active={$mode === 'history'} on:click={() => switchMode('history')}>History</button>
    </div>

    {#if error}<p class="error">{error}</p>{/if}

    {#if $mode === 'search'}
      <div class="search-row">
        <select
          bind:value={$translationId}
          on:change={(e) => handleTranslationChange(parseInt(e.target.value, 10))}
        >
          {#each translations as t, i}
            <option value={t.translationid}>{i + 1} · {t.abbreviation}</option>
          {/each}
        </select>
        <input
          bind:this={searchInput}
          bind:value={query}
          on:keydown={(e) => e.key === 'Enter' && runSearch()}
          placeholder="Jn 3:16 — or a keyword like righteousness"
        />
        <button on:click={runSearch}>Search</button>
      </div>

      <div class="results">
        {#each resultsWithLive as { verse: v, live }}
          <div class="result-card" class:live>
            <div class="result-head">
              <strong>{v.book_name} {v.chapternumber}:{v.versenumber}</strong>
              {#if live}<span class="tally"><i class="dot"></i>ON AIR</span>{/if}
            </div>
            <p>{v.versetext}</p>
            <button on:click={() => project(v)} disabled={live}>
              {live ? 'On screen' : 'Project'}
            </button>
          </div>
        {/each}
        {#if results.length === 0 && query}
          <p class="empty-hint">No results. Try a reference like "Jn 3:16" or a plain keyword.</p>
        {/if}
      </div>
    {/if}

    {#if $mode === 'browse'}
      <div class="browse-crumbs">
        <button class:active={!selectedBook} on:click={() => { selectedBook = null; selectedChapter = null; }}>
          Books
        </button>
        {#if selectedBook}
          <span>/</span>
          <button class:active={selectedBook && !selectedChapter} on:click={() => (selectedChapter = null)}>
            {selectedBook.name}
          </button>
        {/if}
        {#if selectedChapter}
          <span>/</span>
          <span>Chapter {selectedChapter}</span>
        {/if}
      </div>

      {#if !selectedBook}
        <input
          class="book-filter"
          bind:value={bookFilter}
          placeholder="Filter books..."
        />

        <details open>
          <summary>Old Testament ({otBooks.length})</summary>
          <div class="grid">
            {#each otBooks as b}
              <button class="grid-cell" on:click={() => pickBook(b)}>{b.name}</button>
            {/each}
          </div>
        </details>

        <details open>
          <summary>New Testament ({ntBooks.length})</summary>
          <div class="grid">
            {#each ntBooks as b}
              <button class="grid-cell" on:click={() => pickBook(b)}>{b.name}</button>
            {/each}
          </div>
        </details>
      {:else if !selectedChapter}
        {#if chapterCount === 0}
          <p class="empty-hint">No chapters found for this book in the current translation.</p>
        {:else}
          <div class="grid">
            {#each Array(chapterCount) as _, i}
              <button class="grid-cell" on:click={() => pickChapter(i + 1)}>{i + 1}</button>
            {/each}
          </div>
        {/if}
      {:else}
        <div class="results">
          {#each chapterVersesWithLive as { verse: v, live }}
            <div class="result-card" class:live>
              <div class="result-head">
                <strong>{v.book_name} {v.chapternumber}:{v.versenumber}</strong>
                {#if live}<span class="tally"><i class="dot"></i>ON AIR</span>{/if}
              </div>
              <p>{v.versetext}</p>
              <button on:click={() => project(v)} disabled={live}>
                {live ? 'On screen' : 'Project'}
              </button>
            </div>
          {/each}
        </div>
      {/if}
    {/if}

    {#if $mode === 'history'}
      <div class="results">
        {#each sessions as trail}
          {@const root = trail[0]}
          <div class="result-card" class:live={computeLive(root.verse, root.translation_abbr)}>
            <div class="result-head">
              <strong>{root.verse.book_name} {root.verse.chapternumber}:{root.verse.versenumber}</strong>
              <span class="history-time">{root.translation_abbr} · {root.created_at}</span>
            </div>
            <p>{root.verse.versetext}</p>
            <button on:click={() => projectFromHistory(root)} disabled={computeLive(root.verse, root.translation_abbr)}>
              {computeLive(root.verse, root.translation_abbr) ? 'On screen' : 'Project'}
            </button>

            {#if trail.length > 1}
              <div class="trail">
                <span class="trail-label">Visited verses</span>
                {#each trail as entry, i}
                  {#if i > 0 && entry.verse.chapternumber !== trail[i - 1].verse.chapternumber}
                    <span class="trail-divider">||</span>
                  {/if}
                  <button
                    class="trail-chip"
                    class:live={computeLive(entry.verse, entry.translation_abbr)}
                    on:click={() => projectFromHistory(entry)}
                  >
                    {entry.verse.versenumber}
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        {/each}
        {#if historyEntries.length === 0}
          <p class="empty-hint">Nothing projected yet this session.</p>
        {/if}
      </div>
    {/if}
  </div>

  <div class="monitor">
    <div class="monitor-head">
      {#if $livePreview}
        <span class="tally"><i class="dot"></i>ON AIR</span>
      {:else}
        <span class="tally idle"><i class="dot"></i>STANDBY</span>
      {/if}
      <button class="remove-btn" on:click={removeProjection} disabled={!$livePreview || $isPanicked}>
        Remove Projection
      </button>
      <button class="remove-btn" on:click={restoreProjection} disabled={!$isPanicked}>
        Restore
      </button>
    </div>

    <div class="preview-canvas">
      <div class="background-layer" style="{bgStyle} opacity: {background.opacity}"></div>
      {#if background.kind === 'video'}
        <video class="background-layer" src={resolvedBgValue} autoplay loop muted
          style="opacity: {background.opacity}; object-position: {background.position || 'left'} center;"></video>
      {/if}
      <div class="text-layer" class:panicked={$isPanicked}>
        {#if $livePreview}
          <ScriptureDisplay
            verse={$livePreview.verse}
            translationAbbr={$livePreview.translation_abbr}
            fontFamily={displaySettings.fontFamily}
            weightNormal={displaySettings.weightNormal}
            weightWj={displaySettings.weightWj}
            colorNormal={displaySettings.colorNormal}
            colorWj={displaySettings.colorWj}
            colorBracket={displaySettings.colorBracket}
          />
        {:else}
          <p class="empty">Nothing projected</p>
        {/if}
      </div>
    </div>

    {#if $livePreview}
      <div class="monitor-caption">
        {$livePreview.verse.book_name} {$livePreview.verse.chapternumber}:{$livePreview.verse.versenumber}
        <span class="translation-chip">{$livePreview.translation_abbr}</span>
      </div>
    {/if}
  </div>
</div>

<style>
  .console {
    display: grid;
    grid-template-columns: 1fr 460px;
    height: 100vh;
  }

  .workspace {
    padding: 1.5rem;
    overflow-y: auto;
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .toolbar button {
    background: var(--bg-raised);
    color: var(--text-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 0.5rem 0.9rem;
    font-size: 0.85rem;
  }

  .status {
    color: var(--text-muted);
    font-size: 0.8rem;
  }

  .search-row {
    display: flex;
    gap: 0.5rem;
  }

  .search-row select,
  .search-row input {
    background: var(--bg-panel);
    border: 1px solid var(--border);
    color: var(--text-primary);
    border-radius: var(--radius);
    padding: 0.55rem 0.7rem;
    font-size: 0.9rem;
  }

  .search-row input {
    flex: 1;
  }

  .search-row button {
    background: var(--bg-raised);
    color: var(--text-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 0.55rem 1rem;
  }

  .error {
    color: var(--accent-live);
    font-size: 0.85rem;
  }

  .results {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .empty-hint {
    color: var(--text-faint);
    font-size: 0.85rem;
  }

  .result-card {
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 0.9rem 1rem;
  }

  .result-card.live {
    border-color: var(--accent-live-dim);
    background: linear-gradient(180deg, #241a1b, var(--bg-panel) 40%);
  }

  .result-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-family: var(--font-mono);
    font-size: 0.85rem;
    color: var(--text-primary);
  }

  .result-card p {
    margin: 0.5rem 0 0.75rem;
    color: var(--text-muted);
    font-size: 0.9rem;
    line-height: 1.45;
  }

  .result-card button {
    background: var(--bg-raised);
    color: var(--text-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 0.4rem 0.85rem;
    font-size: 0.8rem;
  }

  .result-card button:disabled {
    background: transparent;
    color: var(--text-faint);
    border-color: var(--border);
  }

  .tally {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    font-family: var(--font-mono);
    font-size: 0.7rem;
    letter-spacing: 0.06em;
    color: var(--accent-live);
  }

  .tally.idle {
    color: var(--text-faint);
  }

  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--accent-live);
    box-shadow: 0 0 6px var(--accent-live);
    animation: pulse 1.6s ease-in-out infinite;
  }

  .tally.idle .dot {
    background: var(--text-faint);
    box-shadow: none;
    animation: none;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.35; }
  }

  .monitor {
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    background: var(--bg-panel);
  }

  .preview-canvas {
    position: relative;
    width: 100%;
    aspect-ratio: 16 / 9;
    background: #000;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    overflow: hidden;
    transition: opacity 300ms ease;
  }

  .preview-canvas .background-layer {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .preview-canvas .text-layer {
    position: relative;
    width: 100%;
    height: 100%;
    display: flex;
  }

  .preview-canvas .text-layer.panicked {
    opacity: 0;
  }

  .empty {
    margin: auto;
    color: var(--text-faint);
    font-size: 0.8rem;
  }

  .monitor-caption {
    font-family: var(--font-mono);
    font-size: 0.8rem;
    color: var(--text-muted);
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .translation-chip {
    background: var(--bg-raised);
    border: 1px solid var(--border);
    border-radius: 3px;
    padding: 0.1rem 0.4rem;
    color: var(--text-primary);
    font-size: 0.7rem;
  }

  @media (prefers-reduced-motion: reduce) {
    .dot { animation: none; }
  }

  .monitor-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .remove-btn {
    background: transparent;
    color: var(--text-muted);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 0.3rem 0.7rem;
    font-size: 0.75rem;
  }

  .remove-btn:disabled {
    color: var(--text-faint);
    border-color: var(--border);
    opacity: 0.5;
  }

  .settings-link {
    margin-left: auto;
    color: var(--text-faint);
    font-size: 0.8rem;
    text-decoration: none;
  }

  .mode-tabs {
    display: flex;
    gap: 0.4rem;
    border-bottom: 1px solid var(--border);
    padding-bottom: 0.6rem;
  }

  .mode-tabs button {
    background: transparent;
    color: var(--text-muted);
    border: none;
    padding: 0.4rem 0.8rem;
    border-radius: var(--radius);
    font-size: 0.85rem;
  }

  .mode-tabs button.active {
    background: var(--bg-raised);
    color: var(--text-primary);
  }

  .browse-crumbs {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .browse-crumbs button {
    background: transparent;
    border: none;
    color: var(--text-muted);
    padding: 0.2rem 0.3rem;
    font-size: 0.8rem;
  }

  .browse-crumbs button.active {
    color: var(--text-primary);
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(110px, 1fr));
    gap: 0.5rem;
  }

  .grid-cell {
    background: var(--bg-panel);
    border: 1px solid var(--border);
    color: var(--text-primary);
    border-radius: var(--radius);
    padding: 0.5rem 0.6rem;
    font-size: 0.78rem;
    min-height: 46px;
    display: flex;
    align-items: center;
    justify-content: center;
    text-align: center;
    line-height: 1.25;
  }

  .grid-cell:hover {
    border-color: var(--text-faint);
  }

  .history-time {
    font-size: 0.7rem;
    color: var(--text-faint);
    font-weight: normal;
  }

  .book-filter {
    width: 100%;
    padding: 0.5rem 0.7rem;
    margin-bottom: 0.75rem;
    background: var(--bg-panel);
    border: 1px solid var(--border);
    color: var(--text-primary);
    border-radius: var(--radius);
  }

  details {
    margin-bottom: 0.75rem;
  }

  summary {
    cursor: pointer;
    font-size: 0.85rem;
    color: var(--text-muted);
    padding: 0.4rem 0;
    user-select: none;
  }

  summary:hover {
    color: var(--text-primary);
  }

  .trail {
    margin-top: 0.6rem;
    padding-top: 0.6rem;
    border-top: 1px solid var(--border);
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.35rem;
  }

  .trail-label {
    font-size: 0.7rem;
    color: var(--text-faint);
    margin-right: 0.4rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .trail-chip {
    background: var(--bg-raised);
    border: 1px solid var(--border);
    color: var(--text-muted);
    border-radius: 4px;
    padding: 0.15rem 0.5rem;
    font-size: 0.75rem;
    font-family: var(--font-mono);
  }

  .trail-chip:hover {
    color: var(--text-primary);
    border-color: var(--text-faint);
  }

  .trail-chip.live {
    border-color: var(--accent-live);
    color: var(--accent-live);
  }

  .trail-divider {
    color: var(--text-faint);
    font-size: 0.8rem;
    margin: 0 0.15rem;
  }
</style>
