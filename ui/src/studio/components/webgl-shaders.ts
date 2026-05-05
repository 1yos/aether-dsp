// WebGL shaders for the node graph canvas

export const NODE_VERT = `
  attribute vec2 a_position;
  attribute vec2 a_uv;
  uniform vec2 u_resolution;
  uniform vec2 u_pan;
  uniform float u_zoom;
  varying vec2 v_uv;
  void main() {
    vec2 world = a_position * u_zoom + u_pan;
    vec2 clip = (world / u_resolution) * 2.0 - 1.0;
    gl_Position = vec4(clip.x, -clip.y, 0.0, 1.0);
    v_uv = a_uv;
  }
`;

export const NODE_FRAG = `
  precision mediump float;
  uniform vec4 u_color;
  uniform vec4 u_border_color;
  uniform float u_selected;
  varying vec2 v_uv;
  void main() {
    vec2 q = abs(v_uv - 0.5) - 0.5 + 0.08;
    float d = length(max(q, 0.0)) - 0.08;
    float border = smoothstep(0.0, 0.01, -d) - smoothstep(-0.015, -0.005, -d);
    float fill = smoothstep(0.005, -0.005, d);
    vec4 col = mix(u_color, u_border_color, border * (0.5 + u_selected * 0.5));
    gl_FragColor = col * fill;
  }
`;

export const CABLE_VERT = `
  attribute vec2 a_position;
  attribute float a_t;
  uniform vec2 u_resolution;
  uniform vec2 u_pan;
  uniform float u_zoom;
  varying float v_t;
  void main() {
    vec2 world = a_position * u_zoom + u_pan;
    vec2 clip = (world / u_resolution) * 2.0 - 1.0;
    gl_Position = vec4(clip.x, -clip.y, 0.0, 1.0);
    v_t = a_t;
  }
`;

export const CABLE_FRAG = `
  precision mediump float;
  uniform vec4 u_color;
  uniform float u_time;
  uniform float u_active;
  varying float v_t;
  void main() {
    float flow = fract(v_t - u_time * 0.4);
    float dot = smoothstep(0.0, 0.08, flow) * smoothstep(0.16, 0.08, flow);
    float alpha = 0.5 + dot * u_active * 0.5;
    gl_FragColor = vec4(u_color.rgb, u_color.a * alpha);
  }
`;

export const BG_VERT = `
  attribute vec2 a_position;
  void main() { gl_Position = vec4(a_position, 0.0, 1.0); }
`;

export const BG_FRAG = `
  precision mediump float;
  uniform vec2 u_resolution;
  uniform vec2 u_pan;
  uniform float u_zoom;
  void main() {
    vec2 world = (gl_FragCoord.xy - u_pan) / u_zoom;
    float grid = 32.0;
    vec2 g = mod(world, grid);
    float dot = smoothstep(1.2, 0.0, length(g - grid * 0.5));
    float major = 0.0;
    if (mod(world.x, grid * 4.0) < 1.0 || mod(world.y, grid * 4.0) < 1.0) major = 0.04;
    gl_FragColor = vec4(0.024, 0.055, 0.094, 1.0) + vec4(dot * 0.04 + major);
  }
`;
