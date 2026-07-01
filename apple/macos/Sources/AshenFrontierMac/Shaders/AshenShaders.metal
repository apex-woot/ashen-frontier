#include <metal_stdlib>
using namespace metal;

struct VertexIn {
    float2 position [[attribute(0)]];
    float2 worldPosition [[attribute(1)]];
    float4 color [[attribute(2)]];
    float material [[attribute(3)]];
};

struct VertexOut {
    float4 position [[position]];
    float2 worldPosition;
    float4 color;
    float material;
};

vertex VertexOut vertex_main(VertexIn input [[stage_in]]) {
    VertexOut output;
    output.position = float4(input.position, 0.0, 1.0);
    output.worldPosition = input.worldPosition;
    output.color = input.color;
    output.material = input.material;
    return output;
}

static float terrain_noise(float2 cell) {
    return fract(sin(dot(cell, float2(127.1, 311.7))) * 43758.5453);
}

fragment float4 fragment_main(VertexOut input [[stage_in]]) {
    if (input.material > 0.5) {
        return input.color;
    }

    float2 world = input.worldPosition;
    float2 cell = floor(world);
    float2 local = fract(world);
    float noise = terrain_noise(cell);
    float broadNoise = terrain_noise(floor(world * 0.125));
    float pathBand = smoothstep(0.46, 0.54, sin((world.x + world.y * 0.63) * 0.045) * 0.5 + 0.5);

    float3 grass = float3(0.105, 0.185, 0.135);
    float3 moss = float3(0.155, 0.245, 0.160);
    float3 ash = float3(0.145, 0.155, 0.135);
    float3 color = mix(grass, moss, noise * 0.42 + broadNoise * 0.28);
    color = mix(color, ash, pathBand * 0.18);

    float edgeDistance = min(min(local.x, local.y), min(1.0 - local.x, 1.0 - local.y));
    float pixelWorldSize = max(fwidth(world.x), fwidth(world.y));
    float gridAlpha = 1.0 - smoothstep(0.65, 2.5, pixelWorldSize);
    float lineWidth = max(pixelWorldSize * 0.72, 0.018);
    float gridLine = 1.0 - smoothstep(0.0, lineWidth, edgeDistance);
    color = mix(color, float3(0.045, 0.070, 0.055), gridLine * gridAlpha * 0.55);

    float majorEdgeX = min(fract(world.x * 0.125), 1.0 - fract(world.x * 0.125));
    float majorEdgeY = min(fract(world.y * 0.125), 1.0 - fract(world.y * 0.125));
    float majorX = 1.0 - smoothstep(0.0, lineWidth * 0.24, majorEdgeX);
    float majorY = 1.0 - smoothstep(0.0, lineWidth * 0.24, majorEdgeY);
    float majorGrid = max(majorX, majorY) * (1.0 - smoothstep(2.0, 8.0, pixelWorldSize));
    color = mix(color, float3(0.060, 0.095, 0.075), majorGrid * 0.35);

    return float4(color, 1.0);
}
