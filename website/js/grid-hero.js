/** Wavy perspective grid for home hero  -  grid only, no particles. */
(function () {
  const canvas = document.getElementById("home-grid-canvas");
  if (!canvas) return;

  const ctx = canvas.getContext("2d");
  if (!ctx) return;

  const GRID = 40;
  const GRID_LINE = "rgba(251, 219, 90, 0.28)";

  function resize() {
    const dpr = window.devicePixelRatio || 1;
    const w = window.innerWidth;
    const h = window.innerHeight;
    canvas.width = w * dpr;
    canvas.height = h * dpr;
    canvas.style.width = `${w}px`;
    canvas.style.height = `${h}px`;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    draw(w, h);
  }

  function draw(width, height) {
    ctx.clearRect(0, 0, width, height);
    ctx.strokeStyle = GRID_LINE;
    ctx.lineWidth = 1;

    const centerX = width / 2;
    const centerY = height / 2;

    for (let x = -GRID; x < width + GRID; x += GRID) {
      ctx.beginPath();
      for (let y = 0; y <= height; y += 2) {
        const dist = Math.hypot(x - centerX, y - centerY);
        const wave = Math.sin(dist * 0.02) * 20;
        const perspective = 1 - dist / (width * 0.8);
        const adjustedX = x + wave * Math.max(0, perspective);
        if (y === 0) ctx.moveTo(adjustedX, y);
        else ctx.lineTo(adjustedX, y);
      }
      ctx.stroke();
    }

    for (let y = -GRID; y < height + GRID; y += GRID) {
      ctx.beginPath();
      for (let x = 0; x <= width; x += 2) {
        const dist = Math.hypot(x - centerX, y - centerY);
        const wave = Math.sin(dist * 0.02) * 20;
        const perspective = 1 - dist / (height * 0.8);
        const adjustedY = y + wave * Math.max(0, perspective);
        if (x === 0) ctx.moveTo(x, adjustedY);
        else ctx.lineTo(x, adjustedY);
      }
      ctx.stroke();
    }
  }

  resize();
  window.addEventListener("resize", resize);
})();
