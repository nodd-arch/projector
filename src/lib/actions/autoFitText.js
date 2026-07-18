export function autoFitText(node, fontFamily) {
  function fit() {
    const container = node.parentElement;
    if (!container) return;

    const refSize = Math.max(12, Math.min(40, container.clientHeight * 0.05));
    container.style.setProperty('--ref-font-size', `${refSize}px`);

    let min = 8;
    let max = 200;
    let best = min;

    node.style.fontSize = `${max}px`;

    while (max - min > 1) {
      const mid = Math.floor((min + max) / 2);
      node.style.fontSize = `${mid}px`;

      const fits =
        container.scrollHeight <= container.clientHeight &&
        node.scrollWidth <= container.clientWidth;

      if (fits) {
        best = mid;
        min = mid;
      } else {
        max = mid;
      }
    }

    node.style.fontSize = `${best}px`;
  }

  // Run immediately on mount — no rAF delay — so the very first paint
  // already shows the correctly fitted size instead of a flash of
  // unstyled text that then jumps/resizes a frame later.
  fit();

  let frame;
  const scheduleFit = () => {
    cancelAnimationFrame(frame);
    frame = requestAnimationFrame(fit);
  };

  const observer = new ResizeObserver(scheduleFit);
  observer.observe(node);
  if (node.parentElement) observer.observe(node.parentElement);

  return {
    update() { fit(); }, // also immediate — font/weight changes shouldn't flash either
    destroy() { cancelAnimationFrame(frame); observer.disconnect(); }
  };
}
