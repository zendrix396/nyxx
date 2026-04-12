# SEGMENT 4: WEBGL, SHADERS, AND TEXTURE LAYERING

## 1. Global Texture Overlay (The "Film" Effect)
- **Constraint:** Never use plain CSS backgrounds. Every premium site must have a subtle "film grain" or "noise" overlay.
- **Protocol:** Implement a hidden `<canvas>` or `<svg>` overlay covering the entire viewport.
  - **SVG Approach:** Use a filter `<feTurbulence type="fractalNoise" baseFrequency="0.65" numOctaves="3" stitchTiles="stitch"/>`.
  - **CSS:** `opacity: 0.05; pointer-events: none; z-index: 9999;`
- **Purpose:** This breaks the "plastic" look of standard hex colors and adds organic texture.

## 2. WebGL/Three.js Scene Management
- **Renderer Setup:** Always initialize `THREE.WebGLRenderer` with `{ antialias: true, alpha: true, powerPreference: "high-performance" }`.
- **Memory/Cleanup:**
  - Every WebGL implementation must include a `destroy()` method.
  - Must call `geometry.dispose()`, `material.dispose()`, and `renderer.dispose()` on unmount.
  - Must clear the scene with `scene.traverse()` to dispose of all textures/materials.
- **Responsive Canvas:** Always bind a `resize` event listener to update `camera.aspect` and `renderer.setSize(window.innerWidth, window.innerHeight)`.

## 3. Shader Imperatives (Custom Dynamics)
- **Fluid/Distortion Shaders:** If an animation requires "liquid" or "glass" distortion, use a fragment shader with `texture2D` and a `uMouse` uniform.
  - **The "Glassform" Effect:** Calculate `distortedUV` using `fractalGlass(x)` functions (using multiple iterations of a `displacement` function). 
  - **Parallax Mapping:** Add `parallaxOffset` to `uv` based on `uMouse` position to create depth.
- **Optimization:** Use `requestAnimationFrame` for the render loop. Never execute heavy math inside the `main()` shader function if it can be passed as a uniform.

## 4. Canvas-Based Particle/Fluid Systems
- **Grid Systems:** When handling thousands of particles, use `THREE.DataTexture` for positions. This allows the GPU to handle the heavy lifting of position calculations (e.g., mouse interaction/repulsion).
- **Interactive Force:** 
  - Mouse interaction logic must use a "Lerp-to-Target" approach (not direct mouse position mapping). 
  - Example: `position += (target - position) * easing;`
  - Ensure the canvas is set to `mix-blend-mode: normal` or `screen` depending on the background contrast.

## 5. Shader Boilerplate (The "Standard")
- **Uniforms:** Every shader must accept `uTime` (for animations), `uResolution` (for aspect ratios), `uMouse` (for interactivity), and `uTexture` (for content).
- **Interpolation:** Use `smoothstep` for transition masks.
- **Precision:** ALWAYS declare `precision highp float;` at the top of every shader to ensure visual parity across different displays and browsers.