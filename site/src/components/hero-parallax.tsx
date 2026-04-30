"use client";

import { useEffect, useRef } from "react";
import { withBasePath } from "@/lib/shared";

export function HeroParallax() {
  const rootRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const root = rootRef.current;
    if (!root) return;

    const media = window.matchMedia("(prefers-reduced-motion: reduce)");
    const mobileMedia = window.matchMedia("(max-width: 767px)");
    let frame = 0;
    let scrollShift = 0;
    let gridShift = 0;
    let pointerX = 0;
    let pointerY = 0;

    const applyOffsets = () => {
      root.style.setProperty("--hero-diagram-x", `${Math.round(pointerX * -0.35)}px`);
      root.style.setProperty(
        "--hero-diagram-y",
        `${Math.round(scrollShift * -0.72 + pointerY * -0.35)}px`,
      );
      root.style.setProperty("--hero-grid-x", `${Math.round(pointerX * 0.22)}px`);
      root.style.setProperty(
        "--hero-grid-y",
        `${Math.round(gridShift * -0.45 + pointerY * 0.18)}px`,
      );
    };

    const update = () => {
      frame = 0;
      if (media.matches) {
        scrollShift = 0;
        gridShift = 0;
        pointerX = 0;
        pointerY = 0;
        applyOffsets();
        return;
      }

      const parent = root.parentElement;
      const rect = parent?.getBoundingClientRect();
      const height = rect?.height || window.innerHeight || 1;
      const progress = rect ? Math.min(Math.max(-rect.top / height, 0), 1) : 0;

      const isMobile = mobileMedia.matches;

      scrollShift = progress * (isMobile ? 132 : 96);
      gridShift = progress * (isMobile ? 64 : 44);
      applyOffsets();
    };

    const requestUpdate = () => {
      if (frame === 0) {
        frame = window.requestAnimationFrame(update);
      }
    };

    const onPointerMove = (event: PointerEvent) => {
      if (media.matches) return;

      const rect = root.getBoundingClientRect();
      const isInside =
        event.clientX >= rect.left &&
        event.clientX <= rect.right &&
        event.clientY >= rect.top &&
        event.clientY <= rect.bottom;

      if (!isInside) {
        pointerX = 0;
        pointerY = 0;
        applyOffsets();
        return;
      }

      const x = (event.clientX - rect.left) / rect.width - 0.5;
      const y = (event.clientY - rect.top) / rect.height - 0.5;

      pointerX = x * 26;
      pointerY = y * 18;
      applyOffsets();
    };

    update();
    window.addEventListener("scroll", requestUpdate, { passive: true });
    window.addEventListener("resize", requestUpdate);
    window.addEventListener("pointermove", onPointerMove, { passive: true });
    media.addEventListener("change", requestUpdate);
    mobileMedia.addEventListener("change", requestUpdate);

    return () => {
      if (frame !== 0) window.cancelAnimationFrame(frame);
      window.removeEventListener("scroll", requestUpdate);
      window.removeEventListener("resize", requestUpdate);
      window.removeEventListener("pointermove", onPointerMove);
      media.removeEventListener("change", requestUpdate);
      mobileMedia.removeEventListener("change", requestUpdate);
    };
  }, []);

  return (
    <div ref={rootRef} aria-hidden="true" className="hero-parallax">
      <img
        alt=""
        className="hero-parallax__diagram"
        src={withBasePath("/home-hero/signals-relay-hero-v1.jpg")}
      />
      <div className="hero-parallax__grid" />
      <div className="hero-parallax__shade" />
    </div>
  );
}
