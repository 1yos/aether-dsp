/**
 * WebGL graph renderer — core rendering logic.
 * Manages GL context, compiles shaders, draws nodes and cables.
 */

import { NODE_VERT, NODE_FRAG, CABLE_VERT, CABLE_FRAG, BG_VERT, BG_FRAG } from "./webgl-shaders";

export interface GraphNode {
  id: string;
  x: number;
  y: number;
  width: number;
  height: number;
  color: string;
  label: string;
  selected: boolean;
  isOutput: boolean;
}

export interface GraphEdge {
  id: string;
  srcX: number;
  srcY: number;
  dstX: number;
  dstY: number;
  color: string;
}

function hexToRgba(hex: string, alpha = 1): [number, number, number, number] {
  const r = parseInt(hex.slice(1, 3), 16) / 255;
  const g = parseInt(hex.slice(3, 5), 16) / 255;
  const b = parseInt(hex.slice(5, 7), 16) / 255;
  return [r, g, b, alpha];
}

function compileShader(gl: WebGLRenderingContext, type: number, src: string): WebGLShader {
  const shader = gl.createShader(type)!;
  gl.shaderSource(shader, src);
  gl.compileShader(shader);
  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    throw new Error("Shader compile error: " + gl.getShaderInfoLog(shader));
  }
  return shader;
}

function createProgram(gl: WebGLRenderingContext, vert: string, frag: string): WebGLProgram {
  const prog = gl.createProgram()!;
  gl.attachShader(prog, compileShader(gl, gl.VERTEX_SHADER, vert));
  gl.attachShader(prog, compileShader(gl, gl.FRAGMENT_SHADER, frag));
  gl.linkProgram(prog);
  if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
    throw new Error("Program link error: " + gl.getProgramInfoLog(prog));
  }
  return prog;
}

// Cubic Bezier point at t
function bezier(p0: number, p1: number, p2: number, p3: number, t: number): number {
  const mt = 1 - t;
  return mt * mt * mt * p0 + 3 * mt * mt * t * p1 + 3 * mt * t * t * p2 + t * t * t * p3;
}

export class WebGLRenderer {
  private gl: WebGLRenderingContext;
  private nodeProg: WebGLProgram;
  private cableProg: WebGLProgram;
  private bgProg: WebGLProgram;
  private quadBuf: WebGLBuffer;
  private cableBuf: WebGLBuffer;
  private bgBuf: WebGLBuffer;
  private time = 0;

  constructor(canvas: HTMLCanvasElement) {
    const gl = canvas.getContext("webgl", { antialias: true, alpha: false });
    if (!gl) throw new Error("WebGL not supported");
    this.gl = gl;

    this.nodeProg = createProgram(gl, NODE_VERT, NODE_FRAG);
    this.cableProg = createProgram(gl, CABLE_VERT, CABLE_FRAG);
    this.bgProg = createProgram(gl, BG_VERT, BG_FRAG);

    // Full-screen quad for background
    this.bgBuf = gl.createBuffer()!;
    gl.bindBuffer(gl.ARRAY_BUFFER, this.bgBuf);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([
      -1, -1, 1, -1, -1, 1, 1, 1
    ]), gl.STATIC_DRAW);

    // Reusable quad for nodes (unit square, will be scaled via uniforms)
    this.quadBuf = gl.createBuffer()!;
    gl.bindBuffer(gl.ARRAY_BUFFER, this.quadBuf);
    // position (x,y) + uv (u,v) interleaved
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([
      0, 0, 0, 0,
      1, 0, 1, 0,
      0, 1, 0, 1,
      1, 1, 1, 1,
    ]), gl.STATIC_DRAW);

    this.cableBuf = gl.createBuffer()!;

    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
  }

  resize(w: number, h: number) {
    this.gl.viewport(0, 0, w, h);
  }

  render(
    nodes: GraphNode[],
    edges: GraphEdge[],
    pan: [number, number],
    zoom: number,
    canvasW: number,
    canvasH: number,
    audioActive: boolean,
    dt: number,
  ) {
    const gl = this.gl;
    this.time += dt;

    gl.clearColor(0.024, 0.055, 0.094, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);

    // ── Background grid ───────────────────────────────────────────────────────
    gl.useProgram(this.bgProg);
    const bgPos = gl.getAttribLocation(this.bgProg, "a_position");
    gl.bindBuffer(gl.ARRAY_BUFFER, this.bgBuf);
    gl.enableVertexAttribArray(bgPos);
    gl.vertexAttribPointer(bgPos, 2, gl.FLOAT, false, 0, 0);
    gl.uniform2f(gl.getUniformLocation(this.bgProg, "u_resolution"), canvasW, canvasH);
    gl.uniform2f(gl.getUniformLocation(this.bgProg, "u_pan"), pan[0], pan[1]);
    gl.uniform1f(gl.getUniformLocation(this.bgProg, "u_zoom"), zoom);
    gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);

    // ── Cables ────────────────────────────────────────────────────────────────
    gl.useProgram(this.cableProg);
    const cablePos = gl.getAttribLocation(this.cableProg, "a_position");
    const cableT = gl.getAttribLocation(this.cableProg, "a_t");

    for (const edge of edges) {
      const SEGMENTS = 48;
      const verts: number[] = [];
      const cx1 = edge.srcX + 80;
      const cx2 = edge.dstX - 80;

      for (let i = 0; i <= SEGMENTS; i++) {
        const t = i / SEGMENTS;
        const x = bezier(edge.srcX, cx1, cx2, edge.dstX, t);
        const y = bezier(edge.srcY, edge.srcY, edge.dstY, edge.dstY, t);
        verts.push(x, y, t);
      }

      const data = new Float32Array(verts);
      gl.bindBuffer(gl.ARRAY_BUFFER, this.cableBuf);
      gl.bufferData(gl.ARRAY_BUFFER, data, gl.DYNAMIC_DRAW);

      const stride = 3 * 4;
      gl.enableVertexAttribArray(cablePos);
      gl.vertexAttribPointer(cablePos, 2, gl.FLOAT, false, stride, 0);
      gl.enableVertexAttribArray(cableT);
      gl.vertexAttribPointer(cableT, 1, gl.FLOAT, false, stride, 8);

      gl.uniform2f(gl.getUniformLocation(this.cableProg, "u_resolution"), canvasW, canvasH);
      gl.uniform2f(gl.getUniformLocation(this.cableProg, "u_pan"), pan[0], pan[1]);
      gl.uniform1f(gl.getUniformLocation(this.cableProg, "u_zoom"), zoom);
      gl.uniform1f(gl.getUniformLocation(this.cableProg, "u_time"), this.time);
      gl.uniform1f(gl.getUniformLocation(this.cableProg, "u_active"), audioActive ? 1 : 0);

      const [r, g, b, a] = hexToRgba(edge.color || "#38bdf8", 0.7);
      gl.uniform4f(gl.getUniformLocation(this.cableProg, "u_color"), r, g, b, a);

      gl.lineWidth(2);
      gl.drawArrays(gl.LINE_STRIP, 0, SEGMENTS + 1);
    }

    // ── Nodes ─────────────────────────────────────────────────────────────────
    gl.useProgram(this.nodeProg);
    const nodePos = gl.getAttribLocation(this.nodeProg, "a_position");
    const nodeUv = gl.getAttribLocation(this.nodeProg, "a_uv");

    gl.bindBuffer(gl.ARRAY_BUFFER, this.quadBuf);
    gl.enableVertexAttribArray(nodePos);
    gl.vertexAttribPointer(nodePos, 2, gl.FLOAT, false, 16, 0);
    gl.enableVertexAttribArray(nodeUv);
    gl.vertexAttribPointer(nodeUv, 2, gl.FLOAT, false, 16, 8);

    for (const node of nodes) {
      // Scale the unit quad to node dimensions
      const scaledVerts = new Float32Array([
        node.x,             node.y,              0, 0,
        node.x + node.width, node.y,             1, 0,
        node.x,             node.y + node.height, 0, 1,
        node.x + node.width, node.y + node.height, 1, 1,
      ]);
      gl.bufferData(gl.ARRAY_BUFFER, scaledVerts, gl.DYNAMIC_DRAW);

      gl.uniform2f(gl.getUniformLocation(this.nodeProg, "u_resolution"), canvasW, canvasH);
      gl.uniform2f(gl.getUniformLocation(this.nodeProg, "u_pan"), pan[0], pan[1]);
      gl.uniform1f(gl.getUniformLocation(this.nodeProg, "u_zoom"), zoom);
      gl.uniform1f(gl.getUniformLocation(this.nodeProg, "u_selected"), node.selected ? 1 : 0);

      const [r, g, b] = hexToRgba(node.color || "#4fc3f7");
      gl.uniform4f(gl.getUniformLocation(this.nodeProg, "u_color"), r * 0.08, g * 0.08, b * 0.08, 0.95);

      const borderColor = node.isOutput ? [0, 0.9, 0.63, 1] :
                          node.selected ? [r, g, b, 1] : [r * 0.3, g * 0.3, b * 0.3, 0.8];
      gl.uniform4f(gl.getUniformLocation(this.nodeProg, "u_border_color"), ...borderColor as [number,number,number,number]);

      gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
    }
  }

  destroy() {
    const gl = this.gl;
    gl.deleteProgram(this.nodeProg);
    gl.deleteProgram(this.cableProg);
    gl.deleteProgram(this.bgProg);
    gl.deleteBuffer(this.quadBuf);
    gl.deleteBuffer(this.cableBuf);
    gl.deleteBuffer(this.bgBuf);
  }
}
