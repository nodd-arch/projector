<script>
  import { autoFitText } from '$lib/actions/autoFitText.js';
  import { fade } from 'svelte/transition';
  import { quintOut } from 'svelte/easing';

  export let verse;
  export let translationAbbr;
  export let fontFamily = 'Poppins';
  export let weightNormal = '400';
  export let weightWj = '700';
  export let colorNormal = '#ffffff';
  export let colorWj = '#e63946';
  export let colorBracket = '#ffffff';

  $: bracketStyled = translationAbbr === 'AMP' || translationAbbr === 'AMPC';

  function splitBrackets(text) {
    if (!bracketStyled) return [{ text, bracketed: false }];
    const parts = [];
    const regex = /\[([^\]]+)\]/g;
    let lastIndex = 0;
    let match;
    while ((match = regex.exec(text)) !== null) {
      if (match.index > lastIndex) {
        parts.push({ text: text.slice(lastIndex, match.index), bracketed: false });
      }
      parts.push({ text: match[1], bracketed: true });
      lastIndex = regex.lastIndex;
    }
    if (lastIndex < text.length) {
      parts.push({ text: text.slice(lastIndex), bracketed: false });
    }
    return parts;
  }

  $: hasSegments = verse.segments && verse.segments.length > 0;
  // Changing this key forces Svelte to destroy/recreate the block below,
  // which is what makes the in/out transitions actually fire on verse change.
  $: transitionKey = `${verse.verseid}-${translationAbbr}`;
</script>

{#key transitionKey}
  <div
    class="projection-container"
    style="--font-family: {fontFamily}; --weight-normal: {weightNormal}; --weight-wj: {weightWj};
           --color-normal: {colorNormal}; --color-wj: {colorWj}; --color-bracket: {colorBracket}"
    in:fade={{ duration: 400, easing: quintOut }}
    out:fade={{ duration: 250, easing: quintOut }}
  >
    <div class="verse-wrapper">
      <p class="scripture-text" use:autoFitText={fontFamily}>
        {#if hasSegments}
          {#each verse.segments as seg}
            {#if seg.type === 'text'}
              {#each splitBrackets(seg.text) as part}
                <span class:wj={seg.wj} class:bracket={part.bracketed}>{part.text}</span>
              {/each}
            {:else if seg.type === 'footnote_marker'}
              <sup class="footnote-marker">†</sup>
            {/if}
          {/each}
        {:else}
          {#each splitBrackets(verse.versetext) as part}
            <span class:bracket={part.bracketed}>{part.text}</span>
          {/each}
        {/if}
      </p>

      <p class="reference-line">
        <span class="ref-text">{verse.book_name} {verse.chapternumber}:{verse.versenumber}</span>
        <span class="translation-tag">{translationAbbr}</span>
      </p>
    </div>
  </div>
{/key}

<style>
    .projection-container {
      position: absolute;
      inset: 0;
      width: 100%;
      height: 100%;
      box-sizing: border-box;
      padding: 6% 8%;
      display: flex;
      align-items: center;
      justify-content: center;
      font-family: var(--font-family), serif;
      font-weight: var(--weight-normal);
      color: var(--color-normal);
    }

  .verse-wrapper {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1.4rem;
    text-align: center;
  }

  .scripture-text {
    margin: 0;
    line-height: 1.3;
    text-align: center;
    max-width: 100%;
  }

  .wj {
    color: var(--color-wj);
    font-weight: var(--weight-wj);
  }

  .bracket {
    color: var(--color-bracket);
    opacity: 0.75;
    font-style: italic;
  }

  .footnote-marker {
    font-size: 0.5em;
    opacity: 0.6;
    margin-left: 2px;
  }

  .reference-line {
    margin: 0;
    display: flex;
    align-items: center;
    gap: 0.6rem;
    font-size: var(--ref-font-size, 20px);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    opacity: 0.75;
    flex-shrink: 0;
  }

  .translation-tag {
    padding: 0.15em 0.6em;
    border: 1px solid currentColor;
    border-radius: 999px;
    font-size: 0.7em;
    opacity: 0.9;
  }
</style>
