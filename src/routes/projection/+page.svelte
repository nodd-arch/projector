<script>
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import ScriptureDisplay from '$lib/ScriptureDisplay.svelte';
  import { convertFileSrc } from '@tauri-apps/api/core';

  let currentPayload = null;
  let isPanicked = false;
  let projectionFont = 'Poppins';
  let background = { kind: 'color', value: '#000000', opacity: 1 };

  let unlistenVerse, unlistenClear, unlistenRestore, unlistenFont, unlistenBg;

  onMount(async () => {
    const storedFont = localStorage.getItem('projectionFont');
    if (storedFont) projectionFont = storedFont;
    const storedBg = localStorage.getItem('background');
    if (storedBg) background = JSON.parse(storedBg);

    unlistenVerse = await listen('verse-update', (event) => {
      currentPayload = event.payload;
      isPanicked = false;
    });
    unlistenClear = await listen('panic-clear', () => { isPanicked = true; });
    unlistenRestore = await listen('panic-restore', () => { isPanicked = false; });
    unlistenFont = await listen('font-update', (e) => { projectionFont = e.payload; });
    unlistenBg = await listen('background-update', (e) => {
      background = e.payload;
      localStorage.setItem('background', JSON.stringify(e.payload));
    });

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
  });

  onDestroy(() => {
    unlistenVerse?.(); unlistenClear?.(); unlistenRestore?.();
    unlistenFont?.(); unlistenBg?.();
  });

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

  let displaySettings = {
    fontFamily: 'Poppins',
    weightNormal: '400',
    weightWj: '700',
    colorNormal: '#ffffff',
    colorWj: '#e63946',
    colorBracket: '#ffffff',
  };

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
</script>

<div class="projection-root">
  <div class="background-layer" style="{bgStyle} opacity: {background.opacity}"></div>

  {#if background.kind === 'video'}
    <video class="background-layer" src={resolvedBgValue} autoplay loop muted
      style="opacity: {background.opacity}; object-position: {background.position || 'left'} center;"></video>
  {/if}

  <div class="text-layer" class:panicked={isPanicked}>
    {#if currentPayload}
        <ScriptureDisplay
        verse={currentPayload.verse}
        translationAbbr={currentPayload.translation_abbr}
          fontFamily={displaySettings.fontFamily}
          weightNormal={displaySettings.weightNormal}
          weightWj={displaySettings.weightWj}
          colorNormal={displaySettings.colorNormal}
          colorWj={displaySettings.colorWj}
          colorBracket={displaySettings.colorBracket}
        />
    {/if}
  </div>
</div>

<style>
  .projection-root {
    position: relative;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    background: black;
  }

  .background-layer {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .text-layer {
    position: relative;
    width: 100%;
    height: 100%;
    display: flex;
    transition: opacity 300ms ease;
  }

  .text-layer.panicked {
    opacity: 0;
  }
</style>
