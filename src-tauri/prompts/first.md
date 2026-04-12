# CORE ARCHITECTURE: GSAP & SCROLL PERFORMANCE

## 1. GSAP & ScrollTrigger Initialization
- **Register Plugins:** ALWAYS register `ScrollTrigger`, `SplitText`, `CustomEase`, and `MotionPathPlugin` at the start of every component.
- **Context Management:** Wrap every animation in `useGSAP(..., { scope: containerRef })`. 
- **Memory Safety:** Every `ScrollTrigger` and `GSAP` animation must be killed or reverted on component unmount. 
  - Implementation: `return () => { ScrollTrigger.getAll().forEach(t => t.kill()); };`
- **Ticker Optimization:** Disable GSAP's lag smoothing to ensure animations are frame-perfect: `gsap.ticker.lagSmoothing(0);`

## 2. Smooth Scrolling (Lenis Integration)
- **Root Setup:** Initialize `Lenis` globally. Bind it to GSAP: `lenis.on('scroll', ScrollTrigger.update); gsap.ticker.add((time) => lenis.raf(time * 1000));`.
- **Responsive Settings:** Use a dynamic configuration object (as defined in `client-layout.js` of provided source). `lerp` should be lower (0.05) for mobile and higher (0.1) for desktop.
- **Interaction Prevention:** Always add `data-lenis-prevent` to any scrollable element inside a main Lenis container (like sliders or internal chat windows).

## 3. The Pinned Layout Protocol
- **Pinning:** Use `ScrollTrigger.create({ trigger: ..., pin: true, pinSpacing: true, scrub: 1, ... })`.
- **Pin-Spacing:** If you want a seamless transition between pinned sections without jumps, use `pinSpacing: false` on specific elements, and manually manage the container's height/padding to prevent layout shift.
- **Refresh:** Always call `ScrollTrigger.refresh()` after dynamic content loading (e.g., after images load) to recalculate scroll positions.

## 4. Performance Optimization (The "Jank" Prevention)
- **Will-Change:** Every animated element must have `will-change: transform, opacity;` applied via CSS.
- **Force3D:** Ensure GSAP animations utilize GPU hardware acceleration by setting `force3D: true` on complex transform operations.
- **Passive Listeners:** Always use `{ passive: true }` for scroll and touch listeners.

## 5. Animation Lifecycle Management
- **Initial Load:** Check `sessionStorage` or a shared state to identify the "Initial Load" of the site. Only play complex intros (preloader/staggered hero) if `isInitialLoad` is true.
- **Cleanup:** Never leak animation memory. Every React `useEffect` or `useGSAP` hook that starts an animation must provide a cleanup function that kills the timeline or the specific instances.