import Metal
import MetalKit
import simd

private let gridWidth: Float = 32.0
private let gridHeight: Float = 24.0

struct Vertex {
    var position: SIMD2<Float>
    var color: SIMD4<Float>
}

final class Renderer: NSObject, MTKViewDelegate {
    private let world: RustWorld
    private let commandQueue: MTLCommandQueue
    private let pipelineState: MTLRenderPipelineState

    init(device: MTLDevice, colorPixelFormat: MTLPixelFormat, world: RustWorld) throws {
        guard let commandQueue = device.makeCommandQueue() else {
            throw RendererError.missingCommandQueue
        }

        self.world = world
        self.commandQueue = commandQueue
        self.pipelineState = try Self.makePipelineState(device: device, pixelFormat: colorPixelFormat)
        super.init()
    }

    func mtkView(_ view: MTKView, drawableSizeWillChange size: CGSize) {}

    func draw(in view: MTKView) {
        guard let descriptor = view.currentRenderPassDescriptor,
              let drawable = view.currentDrawable,
              let commandBuffer = commandQueue.makeCommandBuffer(),
              let encoder = commandBuffer.makeRenderCommandEncoder(descriptor: descriptor) else {
            return
        }

        world.step(1)
        var vertices = makeSceneVertices()

        encoder.setRenderPipelineState(pipelineState)
        encoder.setVertexBytes(&vertices, length: vertices.count * MemoryLayout<Vertex>.stride, index: 0)
        encoder.drawPrimitives(type: .triangle, vertexStart: 0, vertexCount: vertices.count)
        encoder.endEncoding()
        commandBuffer.present(drawable)
        commandBuffer.commit()
    }

    private static func makePipelineState(
        device: MTLDevice,
        pixelFormat: MTLPixelFormat
    ) throws -> MTLRenderPipelineState {
        guard let shaderURL = Bundle.module.url(
            forResource: "AshenShaders",
            withExtension: "metal",
            subdirectory: "Shaders"
        ) else {
            throw RendererError.missingShader
        }

        let shaderSource = try String(contentsOf: shaderURL)
        let library = try device.makeLibrary(source: shaderSource, options: nil)

        let descriptor = MTLRenderPipelineDescriptor()
        descriptor.vertexFunction = library.makeFunction(name: "vertex_main")
        descriptor.fragmentFunction = library.makeFunction(name: "fragment_main")
        descriptor.colorAttachments[0].pixelFormat = pixelFormat
        descriptor.vertexDescriptor = makeVertexDescriptor()

        return try device.makeRenderPipelineState(descriptor: descriptor)
    }

    private static func makeVertexDescriptor() -> MTLVertexDescriptor {
        let descriptor = MTLVertexDescriptor()
        descriptor.attributes[0].format = .float2
        descriptor.attributes[0].offset = 0
        descriptor.attributes[0].bufferIndex = 0
        descriptor.attributes[1].format = .float4
        descriptor.attributes[1].offset = MemoryLayout<SIMD2<Float>>.stride
        descriptor.attributes[1].bufferIndex = 0
        descriptor.layouts[0].stride = MemoryLayout<Vertex>.stride
        descriptor.layouts[0].stepFunction = .perVertex
        return descriptor
    }

    private func makeSceneVertices() -> [Vertex] {
        var vertices: [Vertex] = []
        appendQuad(
            center: worldToClip(x: 16.5, y: 12.5),
            halfSize: SIMD2<Float>(0.045, 0.06),
            color: SIMD4<Float>(0.12, 0.34, 0.42, 1.0),
            to: &vertices
        )

        for unit in world.units() {
            appendQuad(
                center: worldToClip(x: unit.x, y: unit.y),
                halfSize: SIMD2<Float>(0.018, 0.024),
                color: SIMD4<Float>(0.76, 0.82, 0.64, 1.0),
                to: &vertices
            )
        }

        for enemy in world.enemies() {
            appendQuad(
                center: worldToClip(x: enemy.x, y: enemy.y),
                halfSize: SIMD2<Float>(0.016, 0.021),
                color: SIMD4<Float>(0.78, 0.18, 0.16, 1.0),
                to: &vertices
            )
        }

        return vertices
    }
}

private func worldToClip(x: Float, y: Float) -> SIMD2<Float> {
    SIMD2<Float>(
        (x / gridWidth) * 2.0 - 1.0,
        (y / gridHeight) * 2.0 - 1.0
    )
}

private func appendQuad(
    center: SIMD2<Float>,
    halfSize: SIMD2<Float>,
    color: SIMD4<Float>,
    to vertices: inout [Vertex]
) {
    let topLeft = SIMD2<Float>(center.x - halfSize.x, center.y + halfSize.y)
    let topRight = SIMD2<Float>(center.x + halfSize.x, center.y + halfSize.y)
    let bottomLeft = SIMD2<Float>(center.x - halfSize.x, center.y - halfSize.y)
    let bottomRight = SIMD2<Float>(center.x + halfSize.x, center.y - halfSize.y)

    vertices.append(Vertex(position: topLeft, color: color))
    vertices.append(Vertex(position: bottomLeft, color: color))
    vertices.append(Vertex(position: topRight, color: color))
    vertices.append(Vertex(position: topRight, color: color))
    vertices.append(Vertex(position: bottomLeft, color: color))
    vertices.append(Vertex(position: bottomRight, color: color))
}

enum RendererError: Error {
    case missingCommandQueue
    case missingShader
}
