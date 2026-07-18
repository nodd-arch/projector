<script>
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { onMount } from 'svelte';

  let tab = 'fonts';
  let systemFonts = [];
  let error = null;
  let bgPosition = 'left';

  const weightOptions = [
    { value: '300', label: 'Light' },
    { value: '400', label: 'Regular' },
    { value: '500', label: 'Medium' },
    { value: '600', label: 'Semibold' },
    { value: '700', label: 'Bold' },
  ];

  const resolutionPresets = [
    { label: '1280 × 720 (HD)', width: 1280, height: 720 },
    { label: '1920 × 1080 (Full HD)', width: 1920, height: 1080 },
    { label: '2560 × 1440 (QHD)', width: 2560, height: 1440 },
    { label: '3840 × 2160 (4K UHD)', width: 3840, height: 2160 },
    { label: '1024 × 768 (4:3)', width: 1024, height: 768 },
    { label: '800 × 600 (4:3)', width: 800, height: 600 },
  ];

  let outputWidth = 1080;
  let outputHeight = 720;

  let fontFamily = 'Poppins';
  let weightNormal = '400';
  let weightWj = '700';
  let colorNormal = '#ffffff';
  let colorWj = '#e63946';
  let colorBracket = '#ffffff';

  let bgKind = 'color';
  let bgValue = '#000000';
  let bgOpacity = 1;

  function loadStored() {
    fontFamily = localStorage.getItem('displayFontFamily') || fontFamily;
    weightNormal = localStorage.getItem('displayWeightNormal') || weightNormal;
    weightWj = localStorage.getItem('displayWeightWj') || weightWj;
    colorNormal = localStorage.getItem('displayColorNormal') || colorNormal;
    colorWj = localStorage.getItem('displayColorWj') || colorWj;
    colorBracket = localStorage.getItem('displayColorBracket') || colorBracket;
    outputWidth = parseInt(localStorage.getItem('outputWidth')) || outputWidth;
    outputHeight = parseInt(localStorage.getItem('outputHeight')) || outputHeight;

    const storedBg = localStorage.getItem('background');
    if (storedBg) {
      const parsed = JSON.parse(storedBg);
      bgKind = parsed.kind;
      bgValue = parsed.value;
      bgOpacity = parsed.opacity;
      bgPosition = parsed.position || 'left';
    }
  }

  async function applyResolution(preset) {
    error = null;
    outputWidth = preset.width;
    outputHeight = preset.height;
    localStorage.setItem('outputWidth', outputWidth);
    localStorage.setItem('outputHeight', outputHeight);
    try {
      await invoke('apply_output_resolution', { width: outputWidth, height: outputHeight });
    } catch (e) {
      error = e;
    }
  }

  onMount(async () => {
    loadStored();
    try {
      systemFonts = await invoke('get_system_fonts');
    } catch (e) {
      error = e;
    }
  });

  async function applyDisplaySettings() {
    error = null;
    localStorage.setItem('displayFontFamily', fontFamily);
    localStorage.setItem('displayWeightNormal', weightNormal);
    localStorage.setItem('displayWeightWj', weightWj);
    localStorage.setItem('displayColorNormal', colorNormal);
    localStorage.setItem('displayColorWj', colorWj);
    localStorage.setItem('displayColorBracket', colorBracket);

    try {
      await invoke('apply_display_settings', {
        settings: {
          font_family: fontFamily,
          weight_normal: weightNormal,
          weight_wj: weightWj,
          color_normal: colorNormal,
          color_wj: colorWj,
          color_bracket: colorBracket,
        },
      });
    } catch (e) {
      error = e;
    }
  }

  async function applyBackground() {
    error = null;
    const cleanValue = bgValue.replace(/^["']|["']$/g, '').trim();
    const config = { kind: bgKind, value: cleanValue, opacity: bgOpacity, position: bgPosition };
    localStorage.setItem('background', JSON.stringify(config));
    try {
      await invoke('apply_background', { config });
    } catch (e) {
      error = e;
    }
  }
</script>

<div class="settings">
  <div class="tabs">
    <button class:active={tab === 'fonts'} on:click={() => (tab = 'fonts')}>Fonts</button>
    <button class:active={tab === 'colors'} on:click={() => (tab = 'colors')}>Colors</button>
    <button class:active={tab === 'background'} on:click={() => (tab = 'background')}>Background</button>
    <button class:active={tab === 'manual'} on:click={() => (tab = 'manual')}>Manual</button>
    <button class:active={tab === 'display'} on:click={() => (tab = 'display')}>Display</button>

    <a class="back" href="/">← Back to console</a>
  </div>

  {#if error}<p class="error">{error}</p>{/if}

  {#if tab === 'fonts'}
    <div class="panel">
      <h2>Projection font</h2>
      <p class="hint">Applies to the projection screen and live preview immediately.</p>

      <div class="field">
        <label for="font-select">Typeface</label>
        <select id="font-select" bind:value={fontFamily} on:change={applyDisplaySettings}>
          {#each systemFonts as f}
            <option value={f}>{f}</option>
          {/each}
        </select>
      </div>

      <div class="field">
        <label for="weight-normal">Weight — normal text</label>
        <select id="weight-normal" bind:value={weightNormal} on:change={applyDisplaySettings}>
          {#each weightOptions as w}
            <option value={w.value}>{w.label}</option>
          {/each}
        </select>
      </div>

      <div class="field">
        <label for="weight-wj">Weight — words of Jesus</label>
        <select id="weight-wj" bind:value={weightWj} on:change={applyDisplaySettings}>
          {#each weightOptions as w}
            <option value={w.value}>{w.label}</option>
          {/each}
        </select>
      </div>

      <p class="sample-label">Preview</p>
      <div class="sample-box">
        <span style="font-family: {fontFamily}; font-weight: {weightNormal}; color: {colorNormal}">
          For God so loved the world
        </span>
        <span style="font-family: {fontFamily}; font-weight: {weightWj}; color: {colorWj}">
          that He gave His only Son
        </span>
      </div>
    </div>
  {/if}

  {#if tab === 'display'}
    <div class="panel">
      <h2>Output resolution</h2>
      <p class="hint">Resizes the projection window immediately if it's already open — this is what you'll share in Google Meet.</p>

      <div class="grid">
        {#each resolutionPresets as preset}
          <button
            class="grid-cell"
            class:active={outputWidth === preset.width && outputHeight === preset.height}
            on:click={() => applyResolution(preset)}
          >
            {preset.label}
          </button>
        {/each}
      </div>
    </div>
  {/if}

  {#if tab === 'colors'}
    <div class="panel">
      <h2>Text colors</h2>
      <p class="hint">Each color applies live to whatever is currently on screen.</p>

      <div class="color-row">
        <label for="color-normal">Normal text</label>
        <input id="color-normal" type="color" bind:value={colorNormal} on:input={applyDisplaySettings} />
        <span class="sample" style="color: {colorNormal}; background: #111; padding: 0.2rem 0.5rem; border-radius: 4px;">Aa</span>
      </div>

      <div class="color-row">
        <label for="color-wj">Words of Jesus</label>
        <input id="color-wj" type="color" bind:value={colorWj} on:input={applyDisplaySettings} />
        <span class="sample" style="color: {colorWj}; background: #111; padding: 0.2rem 0.5rem; border-radius: 4px;">Aa</span>
      </div>

      <div class="color-row">
        <label for="color-bracket">Bracketed / added text</label>
        <input id="color-bracket" type="color" bind:value={colorBracket} on:input={applyDisplaySettings} />
        <span class="sample" style="color: {colorBracket}; background: #111; padding: 0.2rem 0.5rem; border-radius: 4px; font-style: italic;">Aa</span>
      </div>
    </div>
  {/if}

  {#if tab === 'background'}
    <div class="panel">
      <h2>Projection background</h2>
      <p class="hint">Persists through Escape / panic clear — only "Hide Projection Window" removes it.</p>

      <div class="field">
        <label for="bg-kind">Type</label>
        <select id="bg-kind" bind:value={bgKind} on:change={applyBackground}>
          <option value="color">Solid color</option>
          <option value="gradient">Gradient</option>
          <option value="image">Image (file path or URL)</option>
          <option value="video">Video loop (file path or URL)</option>
        </select>
      </div>

      {#if bgKind === 'image' || bgKind === 'video'}
        <div class="field">
          <label for="bg-position">Crop from</label>
          <select id="bg-position" bind:value={bgPosition} on:change={applyBackground}>
            <option value="left">Left</option>
            <option value="center">Center</option>
            <option value="right">Right</option>
          </select>
        </div>
      {/if}

      {#if bgKind === 'color'}
              <div class="field">
                <label for="bg-color">Color</label>
                <input id="bg-color" type="color" bind:value={bgValue} on:input={applyBackground} />
              </div>
            {:else if bgKind === 'gradient'}
              <div class="field">
                <label for="bg-gradient">CSS gradient</label>
                <input
                  id="bg-gradient"
                  type="text"
                  bind:value={bgValue}
                  on:change={applyBackground}
                  placeholder="linear-gradient(135deg, #1a1a2e, #16213e)"
                />
              </div>
            {:else}
              <div class="field">
                <label for="bg-path">{bgKind === 'image' ? 'Image path or URL' : 'Video path or URL'}</label>
                <input
                  id="bg-path"
                  type="text"
                  bind:value={bgValue}
                  on:change={applyBackground}
                  placeholder="C:\path\to\file or https://..."
                />
              </div>

              <div class="field">
                <label for="bg-position">Crop from</label>
                <select id="bg-position" bind:value={bgPosition} on:change={applyBackground}>
                  <option value="left">Left</option>
                  <option value="center">Center</option>
                  <option value="right">Right</option>
                </select>
              </div>
            {/if}

            <div class="field">
              <label for="bg-opacity">Opacity — {Math.round(bgOpacity * 100)}%</label>
              <input
                id="bg-opacity"
                type="range"
                min="0"
                max="1"
                step="0.05"
                bind:value={bgOpacity}
                on:input={applyBackground}
              />
            </div>
          </div>
        {/if}

  {#if tab === 'manual'}
    <div class="panel">
      <h2>Keyboard controls</h2>
      <dl>
        <dt><kbd>Ctrl</kbd>+<kbd>F</kbd></dt>
        <dd>Focus and select the search field, from anywhere</dd>
        <dt><kbd>Tab</kbd></dt>
        <dd>Cycle to the next translation, updates the live projection immediately</dd>
        <dt><kbd>1</kbd>–<kbd>9</kbd></dt>
        <dd>Jump directly to a translation by its position in the dropdown</dd>
        <dt><kbd>←</kbd> / <kbd>→</kbd></dt>
        <dd>Step to the previous or next verse, rolling into the next chapter at the edges</dd>
        <dt><kbd>Esc</kbd></dt>
        <dd>Fade the projection text to black; press again to restore. Background stays untouched.</dd>
      </dl>

      <h2>Workflow</h2>
      <ol>
        <li>Click <strong>Open Projection Window</strong> once at the start of a service — position and share this window in Google Meet, then leave it alone.</li>
        <li>Search a reference or keyword, then click <strong>Project</strong> to put a specific verse on screen.</li>
        <li>Use <kbd>←</kbd>/<kbd>→</kbd> to step through the passage, and <kbd>Tab</kbd> or number keys to switch translations, without touching the mouse.</li>
        <li>Use <strong>Remove Projection</strong> or <kbd>Esc</kbd> to clear the text between segments of the service — the background stays until you close the window.</li>
      </ol>
    </div>
  {/if}
</div>

<style>
  .settings {
    padding: 1.5rem;
    max-width: 720px;
    margin: 0 auto;
  }

  .tabs {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 1.5rem;
    border-bottom: 1px solid var(--border);
    padding-bottom: 0.75rem;
  }

  .tabs button {
    background: transparent;
    color: var(--text-muted);
    border: none;
    padding: 0.4rem 0.8rem;
    border-radius: var(--radius);
    font-size: 0.85rem;
  }

  .tabs button.active {
    background: var(--bg-raised);
    color: var(--text-primary);
  }

  .back {
    margin-left: auto;
    color: var(--text-faint);
    font-size: 0.8rem;
    text-decoration: none;
  }

  .back:hover {
    color: var(--text-muted);
  }

  .panel h2 {
    font-size: 1rem;
    color: var(--text-primary);
    margin: 0 0 0.4rem;
  }

  .hint {
    color: var(--text-faint);
    font-size: 0.82rem;
    margin: 0 0 1rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    margin-bottom: 1rem;
  }

  .field label {
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  select, input[type="text"] {
    background: var(--bg-panel);
    border: 1px solid var(--border);
    color: var(--text-primary);
    border-radius: var(--radius);
    padding: 0.5rem 0.7rem;
    font-size: 0.85rem;
  }

  input[type="color"] {
    width: 48px;
    height: 32px;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: none;
  }

  input[type="range"] {
    width: 100%;
  }

  .color-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 0.9rem;
  }

  .color-row label {
    width: 160px;
    font-size: 0.85rem;
    color: var(--text-muted);
  }

  .sample-label {
    font-size: 0.75rem;
    color: var(--text-faint);
    margin: 1rem 0 0.4rem;
  }

  .sample-box {
    background: #000;
    border-radius: var(--radius);
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    font-size: 1.1rem;
  }

  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.5rem 1rem;
    font-size: 0.85rem;
  }

  dt {
    color: var(--text-primary);
    white-space: nowrap;
  }

  dd {
    margin: 0;
    color: var(--text-muted);
  }

  ol {
    color: var(--text-muted);
    font-size: 0.88rem;
    line-height: 1.6;
    padding-left: 1.2rem;
  }

  kbd {
    font-family: var(--font-mono);
    background: var(--bg-raised);
    border: 1px solid var(--border);
    border-radius: 3px;
    padding: 0.05rem 0.35rem;
    color: var(--text-muted);
    font-size: 0.75rem;
  }

  .error {
    color: var(--accent-live);
    font-size: 0.85rem;
  }

  .grid-cell.active {
    border-color: var(--accent-live);
    color: var(--text-primary);
  }
</style>
