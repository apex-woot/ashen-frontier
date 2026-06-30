#include <metal_stdlib>
using namespace metal;

struct VertexIn {
    float2 position [[attribute(0)]];
    float4 color [[attribute(1)]];
};

struct VertexOut {
    float4 position [[position]];
    float4 color;
};

vertex VertexOut vertex_main(VertexIn input [[stage_in]]) {
    VertexOut output;
    output.position = float4(input.position, 0.0, 1.0);
    output.color = input.color;
    return output;
}

fragment float4 fragment_main(VertexOut input [[stage_in]]) {
    return input.color;
}
