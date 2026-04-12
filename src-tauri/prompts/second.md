# SEGMENT 2: TYPOGRAPHY, EDITORIAL LAYOUTS & MASKING

## 1. The "Editorial" Layout Protocol
- **Grid Strategy:** Move away from standard 12-column frameworks. Use CSS Grid with variable column widths based on the design intent (e.g., 2fr, 1fr, 3fr).
- **Whitespace usage:** Implement "negative space" as a design element. Large sections of empty space (`whitespace-300`, `whitespace-100`) are mandatory to let cinematic images breathe.
- **Asymmetric Stacking:** For project layouts, use absolute positioning or negative margins (`left: -10vw`, `top: -5svh`) to break the rigid flow. Overlap images with text containers.
- **Constraint Handling:** Use `max-width` on typography containers to ensure readability (optimal line length: 60-80 characters).

## 2. Advanced Typography (The "Reveal" Logic)
- **SplitText Strategy:** Never animate raw strings. 
  - ALWAYS use `SplitType` to break headings into `chars` or `lines`.
  - Wrap these in mask `div`s with `overflow: hidden`.
  - Implementation:
    ```javascript
    const split = new SplitType('.title', { types: 'lines, chars', mask: 'lines' });
    gsap.set(split.chars, { y: '110%' }); // Hide initially
    gsap.to(split.chars, { y: '0%', duration: 1.2, stagger: 0.05, ease: 'hop' });
    ```
- **Dynamic Variable Fonts:** Favor variable fonts (like "Geist Mono", "Neue Montreal"). Adjust `font-weight` and `font-style` (italic) dynamically based on interaction states using CSS variables or GSAP `set`.
- **Responsive Clamping:** Always use `clamp()` for font sizes to avoid media query bloat. 
  - Pattern: `font-size: clamp(2.5rem, 1.8rem + 5vw, 9rem);`

## 3. Masking & Reveal Dynamics (Clip-Path Physics)
- **Masking:** Never use simple `display: none` or opacity fades. Everything must "emerge" or "cut."
- **Clip-Path Transitions:**
  - Use `clip-path: polygon(0% 100%, 100% 100%, 100% 100%, 0% 100%)` for initial hidden state.
  - Animate to `clip-path: polygon(0% 100%, 100% 100%, 100% 0%, 0% 0%)` for a reveal that feels like the element is growing out of the scroll line.
- **Dynamic Masking:** For complex interactive text, apply a `background-clip: text` effect combined with linear gradients to simulate light hitting the letters.

## 4. The "Staggered Scramble" (Signature Codegrid Pattern)
- **Scramble Effect:** When revealing headers on hover or scroll, use a custom character scrambler. 
  - Logic: Iteratively replace the `textContent` of a character `<span>` with random characters from a `CHAR` set (e.g., `ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()`) before settling on the target character.
  - Stagger: Apply a delay of `index * 0.05` to the reveal for a high-end "digital-glitch" aesthetic.

## 5. Responsive Fluidity (Mobile-First Constraints)
- **Breakpoints:** Do not define fixed tablet breakpoints. If a design breaks at 1024px, adapt the grid automatically using `flex-wrap: wrap` or shifting from `display: flex` to `flex-direction: column`.
- **Mobile-Specific Motion:** On devices with `width < 1000px`, simplify complex WebGL/Canvas interactions. If a parallax effect is too heavy, revert to standard CSS transition offsets to ensure 60fps performance on low-power mobile devices.