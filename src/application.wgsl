/*
Tasks:
1. Bring colors to the screen by editing your fragment shader.
2. Change the vertices of our two triangles so they cover the whole screen.
3. Experiment with different colors based on the coordinates of a pixel.
   You can, for example, create gradients by dividing the pixel's position
   by the width/height of our surface. Unfortunately, we can't dynamically
   access the width and height from our shaders yet, so you must use hardcoded
   values, e.g.:
     let width = 1280.0;
     let height = 720.0;

   You could also try plotting a function, by comparing the Y-coordinate against
   the value of some f(x). Beware floating point accuracy, you will need some tolerance
   in your comparisons.
   The distance of your Y towards the real result could also be visualized by adjusting
   the intensity of your returned color (the lower the components, the darker the color).

If you have semantic or syntactical errors in your shader, the application will crash on launch.
Scroll past the panic's stack trace to see the actual errors.

These resources may help you with your tasks:
- https://google.github.io/tour-of-wgsl/ (don't mind the "WebGPU is not supported in this browser")
- https://webgpufundamentals.org/webgpu/lessons/webgpu-wgsl-function-reference.html
*/

// two triangles (not yet) covering the screen
const positions: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1, -1), // bottom left
    vec2<f32>(1, -1), // bottom right
    vec2<f32>(1, 1), // top right

    vec2<f32>(1, 1), // top right
    vec2<f32>(-1, 1), // top left
    vec2<f32>(-1, -1), // bottom left
);

@vertex
fn vs_main(
    // We specify 6 vertices and 1 instance in our render pass draw call
    // Our shader is called six times, once for each vertex.
    // As we are not rendering actual meshes from these vertices, it's sufficient to
    // hardcode their positions so the two resulting triangles cover the whole surface.
    @builtin(vertex_index) in_vertex_index: u32
) -> @builtin(position) vec4<f32> {
    // Vertex coordinates are X Y Z W, where the visible screen
    // for X and Y ranges from -1.0 to 1.0.
    // Z is only relevant for depth testing, which we won't cover today.
    // W is only relevant for homogeneous coordinates, which we won't cover today (leave at 1.0)
    return vec4<f32>(positions[in_vertex_index], 0.0, 1.0);
}

@fragment
fn fs_main(
    // When our triangles cover the whole surface, our fragment shader is called
    // for every pixel of our window.
    // The coordinates are in screen space, meaning X and Y range from 0.0 to the width/height
    // of our surface.
    // With our initial window size, that means X ranges from 0 to 1280, and Y from 0 to 720.
    // You can access a vector's individual components as position.x, position.y, ...
    @builtin(position) position: vec4<f32>
) -> @location(0) vec4<f32> {
    // We return a color value in RGBA format, where every component ranges from 0.0 to 1.0.
    return vec4<f32>(position.x / 1280, position.y / 720, 1, 1.0);
}
