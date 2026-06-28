"use client";

import { useEffect, useRef } from "react";

type Node = { x: number; y: number; vx: number; vy: number; r: number };

/**
 * A slow, ambient accretion field. Drifting nodes are drawn toward a softly
 * wandering gravitational center — a "black hole" — and fall in along tangential
 * orbits, growing denser, brighter and bluer near the core where they read as an
 * accretion disk. Nodes still link to nearby neighbors (the agent-mesh metaphor)
 * and re-enter from a screen edge once consumed, keeping the density steady.
 * Rendered in the electric-blue / violet accents on pure black. Honors
 * prefers-reduced-motion, pauses when the tab is hidden, stays pointer-transparent.
 */
export function ParticleField({ className }: { className?: string }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

    let width = 0;
    let height = 0;
    let dpr = Math.min(window.devicePixelRatio || 1, 2);
    let nodes: Node[] = [];
    let raf = 0;
    let running = true;
    let t = 0; // frame counter, drives the black hole's slow wander

    const LINK_DIST = 116;
    const GRAV = 380; // inward pull strength
    const SOFT = 600; // softening so the core stays finite, never explodes
    const SWIRL = 1.6; // tangential push → particles orbit and spiral in
    const DAMP = 0.99; // light drag, so orbits decay gracefully inward
    const MAX_V = 1.7; // speed clamp keeps the rare core-diver from slingshotting

    // Black-hole center, recomputed each frame from the wander path.
    let bhX = 0;
    let bhY = 0;

    const blackHole = () => {
      // A slow Lissajous drift around the canvas center (~2 min per axis cycle),
      // so the well reads as wandering, never darting.
      bhX = width / 2 + Math.sin(t * 0.0009) * width * 0.22;
      bhY = height / 2 + Math.sin(t * 0.00067 + 1.3) * height * 0.2;
    };

    // Re-enter from a random point on the visible rim, moving mostly tangentially
    // so the node begins a fresh inward spiral rather than a straight plunge —
    // matter appears to stream in from every side, keeping the frame full.
    const spawnAtEdge = (n: Node) => {
      const m = 40; // start just beyond the edge
      const side = Math.floor(Math.random() * 4);
      if (side === 0) {
        n.x = Math.random() * width;
        n.y = -m;
      } else if (side === 1) {
        n.x = width + m;
        n.y = Math.random() * height;
      } else if (side === 2) {
        n.x = Math.random() * width;
        n.y = height + m;
      } else {
        n.x = -m;
        n.y = Math.random() * height;
      }
      const d = Math.hypot(bhX - n.x, bhY - n.y) || 1;
      const speed = 0.35 + Math.random() * 0.5;
      n.vx = (-(bhY - n.y) / d) * speed; // tangential = radial rotated 90°
      n.vy = ((bhX - n.x) / d) * speed;
      n.r = Math.random() * 1.4 + 0.6;
    };

    const seed = () => {
      // Density scales with area — denser than a plain starfield, but capped so it
      // stays subtle and cheap (links are O(n²)).
      const count = Math.min(120, Math.round((width * height) / 9000));
      blackHole();
      nodes = Array.from({ length: count }, () => {
        // Start spread across the canvas, already orbiting, so the disk reads as
        // established from the very first frame.
        const x = Math.random() * width;
        const y = Math.random() * height;
        const d = Math.hypot(bhX - x, bhY - y) || 1;
        const speed = 0.3 + Math.random() * 0.5;
        return {
          x,
          y,
          vx: (-(bhY - y) / d) * speed,
          vy: ((bhX - x) / d) * speed,
          r: Math.random() * 1.4 + 0.6,
        };
      });
    };

    const resize = () => {
      const parent = canvas.parentElement;
      if (!parent) return;
      width = parent.clientWidth;
      height = parent.clientHeight;
      dpr = Math.min(window.devicePixelRatio || 1, 2);
      canvas.width = Math.floor(width * dpr);
      canvas.height = Math.floor(height * dpr);
      canvas.style.width = `${width}px`;
      canvas.style.height = `${height}px`;
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      seed();
      if (reduce) draw(); // one static frame
    };

    const draw = () => {
      ctx.clearRect(0, 0, width, height);

      // The well itself: a dark core ringed by a faint blue→violet halo. Low alpha,
      // so it suggests gravity without dominating the canvas.
      const haloR = Math.min(width, height) * 0.42;
      const halo = ctx.createRadialGradient(bhX, bhY, haloR * 0.12, bhX, bhY, haloR);
      halo.addColorStop(0, "rgba(0, 0, 0, 0)");
      halo.addColorStop(0.42, "rgba(59, 158, 255, 0.1)");
      halo.addColorStop(0.62, "rgba(146, 129, 247, 0.06)");
      halo.addColorStop(1, "rgba(0, 0, 0, 0)");
      ctx.fillStyle = halo;
      ctx.beginPath();
      ctx.arc(bhX, bhY, haloR, 0, Math.PI * 2);
      ctx.fill();

      // Links first, so nodes sit on top.
      for (let i = 0; i < nodes.length; i++) {
        for (let j = i + 1; j < nodes.length; j++) {
          const a = nodes[i];
          const b = nodes[j];
          const dx = a.x - b.x;
          const dy = a.y - b.y;
          const dist = Math.hypot(dx, dy);
          if (dist < LINK_DIST) {
            const k = 1 - dist / LINK_DIST;
            ctx.strokeStyle = `rgba(59, 158, 255, ${k * 0.14})`;
            ctx.lineWidth = 1;
            ctx.beginPath();
            ctx.moveTo(a.x, a.y);
            ctx.lineTo(b.x, b.y);
            ctx.stroke();
          }
        }
      }

      // Nodes — brighter and bluer the closer they fall to the core (heating up),
      // softening to violet out in the disk.
      const heatR = Math.min(width, height) * 0.32;
      for (const n of nodes) {
        const heat = Math.max(0, 1 - Math.hypot(n.x - bhX, n.y - bhY) / heatR);
        const r = Math.round(146 + heat * 4);
        const g = Math.round(129 + heat * 71);
        const b = Math.round(247 + heat * 8);
        ctx.beginPath();
        ctx.arc(n.x, n.y, n.r + heat * 0.6, 0, Math.PI * 2);
        ctx.fillStyle = `rgba(${r}, ${g}, ${b}, ${0.45 + heat * 0.45})`;
        ctx.fill();
      }
    };

    const step = () => {
      if (!running) return;
      t += 1;
      blackHole();

      for (const n of nodes) {
        const dx = bhX - n.x;
        const dy = bhY - n.y;
        const d2 = dx * dx + dy * dy;
        const d = Math.sqrt(d2) || 1;
        const ux = dx / d;
        const uy = dy / d;

        // Softened gravity inward + a tangential swirl that strengthens near the
        // core → an orbit that slowly spirals in.
        const g = GRAV / (d2 + SOFT);
        const sw = SWIRL / d;
        n.vx += ux * g - uy * sw;
        n.vy += uy * g + ux * sw;
        n.vx *= DAMP;
        n.vy *= DAMP;

        const sp = Math.hypot(n.vx, n.vy);
        if (sp > MAX_V) {
          n.vx = (n.vx / sp) * MAX_V;
          n.vy = (n.vy / sp) * MAX_V;
        }

        n.x += n.vx;
        n.y += n.vy;

        // Consumed at the core, or drifted well off-canvas → re-enter from an edge.
        const nd = Math.hypot(bhX - n.x, bhY - n.y);
        const off = 120;
        if (
          nd < 16 ||
          n.x < -off ||
          n.x > width + off ||
          n.y < -off ||
          n.y > height + off
        )
          spawnAtEdge(n);
      }

      draw();
      raf = requestAnimationFrame(step);
    };

    const onVisibility = () => {
      if (document.hidden) {
        running = false;
        cancelAnimationFrame(raf);
      } else if (!reduce && !running) {
        running = true;
        raf = requestAnimationFrame(step);
      }
    };

    const ro = new ResizeObserver(resize);
    if (canvas.parentElement) ro.observe(canvas.parentElement);

    resize();
    if (!reduce) raf = requestAnimationFrame(step);
    document.addEventListener("visibilitychange", onVisibility);

    return () => {
      cancelAnimationFrame(raf);
      ro.disconnect();
      document.removeEventListener("visibilitychange", onVisibility);
    };
  }, []);

  return <canvas ref={canvasRef} aria-hidden className={className} />;
}
