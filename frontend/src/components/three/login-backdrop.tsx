"use client"

import * as React from "react"
import * as THREE from "three"

/**
 * A lightweight animated three.js backdrop: two slowly counter-rotating
 * wireframe icosahedrons in brand blues over a drifting particle field.
 * Transparent canvas so the panel's gradient shows through.
 */
export function LoginBackdrop() {
  const containerRef = React.useRef<HTMLDivElement>(null)

  React.useEffect(() => {
    const container = containerRef.current
    if (!container) return

    const prefersReduced = window.matchMedia(
      "(prefers-reduced-motion: reduce)"
    ).matches

    const scene = new THREE.Scene()
    const camera = new THREE.PerspectiveCamera(50, 1, 0.1, 100)
    camera.position.z = 4.2

    const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true })
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2))
    renderer.domElement.style.width = "100%"
    renderer.domElement.style.height = "100%"
    renderer.domElement.style.display = "block"
    container.appendChild(renderer.domElement)

    // Inner (dark blue) + outer (light blue) wireframe shells.
    const innerGeo = new THREE.IcosahedronGeometry(1.3, 1)
    const innerMat = new THREE.MeshBasicMaterial({
      color: 0x1d4ed8,
      wireframe: true,
      transparent: true,
      opacity: 0.55,
    })
    const inner = new THREE.Mesh(innerGeo, innerMat)
    scene.add(inner)

    const outerGeo = new THREE.IcosahedronGeometry(1.8, 1)
    const outerMat = new THREE.MeshBasicMaterial({
      color: 0x7dd3fc,
      wireframe: true,
      transparent: true,
      opacity: 0.35,
    })
    const outer = new THREE.Mesh(outerGeo, outerMat)
    scene.add(outer)

    // Ambient particle field.
    const count = 450
    const positions = new Float32Array(count * 3)
    for (let i = 0; i < count * 3; i++) {
      positions[i] = (Math.random() - 0.5) * 13
    }
    const pointsGeo = new THREE.BufferGeometry()
    pointsGeo.setAttribute("position", new THREE.BufferAttribute(positions, 3))
    const pointsMat = new THREE.PointsMaterial({
      color: 0x60a5fa,
      size: 0.03,
      transparent: true,
      opacity: 0.6,
    })
    const points = new THREE.Points(pointsGeo, pointsMat)
    scene.add(points)

    function resize() {
      const w = container!.clientWidth
      const h = container!.clientHeight
      if (!w || !h) return
      renderer.setSize(w, h, false)
      camera.aspect = w / h
      camera.updateProjectionMatrix()
    }
    resize()

    const resizeObserver = new ResizeObserver(resize)
    resizeObserver.observe(container)

    const clock = new THREE.Clock()
    let frame = 0

    function render() {
      const t = clock.getElapsedTime()
      inner.rotation.x = t * 0.15
      inner.rotation.y = t * 0.2
      outer.rotation.x = -t * 0.1
      outer.rotation.y = -t * 0.12
      points.rotation.y = t * 0.03
      renderer.render(scene, camera)
    }

    function tick() {
      render()
      frame = requestAnimationFrame(tick)
    }

    if (prefersReduced) {
      render()
    } else {
      tick()
    }

    return () => {
      cancelAnimationFrame(frame)
      resizeObserver.disconnect()
      innerGeo.dispose()
      outerGeo.dispose()
      pointsGeo.dispose()
      innerMat.dispose()
      outerMat.dispose()
      pointsMat.dispose()
      renderer.dispose()
      if (renderer.domElement.parentNode === container) {
        container.removeChild(renderer.domElement)
      }
    }
  }, [])

  return <div ref={containerRef} className="absolute inset-0" aria-hidden="true" />
}
