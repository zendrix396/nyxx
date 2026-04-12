# SEGMENT 3: MOTION PHYSICS, CUSTOM EASING & INTERACTION LOGIC

## 1. The Agency Easing Matrix (CRITICAL)
- **Standard Easing is Banned:** Never use default `power1.out` or `linear`.
- **The "Hop" Easing (Standard):** Register `CustomEase` for the signature agency "snappy" reveal.
  - Pattern: `CustomEase.create("hop", "0.9, 0, 0.1, 1");`
- **The "Heavy Drop" Easing:** Use for elements that need to feel like they carry weight (like large container intros).
  - Pattern: `CustomEase.create("hop2", "M0,0 C0.071,0.505 0.192,0.726 0.318,0.852 0.45,0.984 0.504,1 1,1");`

## 2. Magnetic Interaction Protocol
- **Component Anatomy:** Every "magnetic" button or link MUST be a composite of two layers:
  - `Inner Layer`: Contains the text/icon, positioned centrally.
  - `Wrapper Layer`: The "hit-box," sized 100px+ larger than the inner content, usually hidden or transparent.
- **Physics-Based Movement:** Do not use CSS hover transitions. Use GSAP `quickTo` for instantaneous tracking.
  - Logic: Capture mouse delta relative to the wrapper center. Apply `gsap.to(innerLayer, { x: deltaX * 0.5, y: deltaY * 0.5, ease: "power3.out", duration: 0.35 });`
- **Spring-Back:** On `mouseleave`, reset coordinates to `x: 0, y: 0` with a high-friction ease like `ease: "elastic.out(1, 0.5)"` for a "boing" feel.

## 3. Advanced Parallax & Scroll-Drive
- **Lerp (Linear Interpolation) is the Law:** Avoid direct event-to-property mapping. Always use Lerp math for smoothness:
  - `currentX = lerp(currentX, targetX, lerpFactor);`
  - Recommended `lerpFactor` for desktop: `0.05` to `0.1`.
- **Speed Multipliers:** Implement dynamic speed multipliers for images based on viewport scroll velocity.
  - Implementation: Use `ScrollTrigger.update()` inside Lenis's `onScroll` hook to drive animations, NOT window scroll events.
- **Pinning and Perspective:** For card-stacking or parallax-reveal effects, use `transform-style: preserve-3d;` with `perspective: 1000px;`. Combine with GSAP `z` translation for "depth-based" scaling.

## 4. The "Glitch" & Scramble Logic
- **Visual Integrity:** Use a character set for text scrambles: `ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()`.
- **Scramble Loop:** Create a function `scrambleText(element)` that runs for a fixed number of iterations before resolving to the original string. 
- **Stagger:** When scrambling entire lines, use a `stagger: 0.02` between characters so the text resolves left-to-right or in randomized waves.

## 5. Micro-Interaction Polish
- **Weighted Motion:** All interactive elements (cards, images) must have a `will-change: transform;` and use a slight `scale(0.98)` on `mousedown` to provide tactile feedback.
- **Trigger Sensitivity:** Ensure that trigger areas for interactions are generous. Users should not have to be "pixel-perfect" to trigger an effect.