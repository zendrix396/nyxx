# SEGMENT 5: UX IDENTITY, TRANSITIONS & POLISH

## 1. The "Native-Feel" Preloader Protocol
- **Constraint:** Do not show content until 100% of the viewport assets are ready.
- **The Counter Logic:** Always include a visual counter (`00` to `100`).
- **Animation Logic:** Use a timeline:
  1. Increment the counter with random `jump` intervals rather than a fixed linear time.
  2. Animate a `clip-path` reveal (e.g., `polygon(0% 0%, 100% 0%, 100% 0%, 0% 0%)` -> `100% 100%`).
  3. Ensure the preloader hides `pointer-events: none` before removing it from the DOM.
- **State Management:** Use `sessionStorage.setItem('preloaderSeen', 'true')` to prevent the preloader from running on every page navigation.

## 2. Page Transitions (The "View-Transition" API)
- **Fluid Routing:** Use `next-view-transitions` or `framer-motion` `AnimatePresence`. 
- **The Mask Reveal:** Every page transition must involve a masking animation (e.g., curtains, shrinking squares, or clip-path reveals).
- **The Protocol:**
  1. Trigger exit animation (e.g., `scaleY: 1` curtain over the viewport).
  2. Wait for router transition.
  3. Trigger enter animation (e.g., `scaleY: 0` curtain reveal).
  4. Instant `window.scrollTo(0, 0)` on route change to maintain scroll context.

## 3. Responsive Adaptivity (Mobile-First Polish)
- **Responsive Logic:** Never rely solely on CSS `@media` queries for complex animations. Use `gsap.matchMedia()` inside `useGSAP` to build entirely different animation paths for desktop vs mobile.
- **Mobile Interaction:** 
  - On mobile, disable hover-based logic (`mouseenter`/`mouseleave`) and replace with touch-based interactions (tap/swipe).
  - Simplify complex 3D scenes (e.g., reduce particle count, disable WebGL trails) for better performance on battery-constrained devices.
- **Touch Targets:** Ensure interactive elements (buttons, magnetic hit-boxes) have a minimum touch area of `44px x 44px`.

## 4. The "Final 5%" Polish
- **Magnetic Buttons:** Apply a subtle `scale(1.05)` and easing to buttons as they approach the cursor.
- **System Indicators:** Include a subtle "System Operational" status (e.g., a small pulsing dot in the footer) to reinforce the "instrument" feel.
- **Scroll Progress:** Always include a global progress indicator (a thin line at the top or a vertical line at the side) to provide visual feedback for long-scrolling pages.

## 5. The "Forbidden" List (Do Not Use)
- No standard browser default cursors; hide them or replace with custom interactive dots/rings.
- No harsh linear color transitions; use easing-informed gradients.
- No `window.alert()` or basic browser prompts.
- No "lazy" loading that causes layout shifts; calculate the space before images load.