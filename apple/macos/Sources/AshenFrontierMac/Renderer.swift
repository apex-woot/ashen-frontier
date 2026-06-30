import Metal
import MetalKit
import simd

struct Vertex {
    var position: SIMD2<Float>
    var color: SIMD4<Float>
}

final class Renderer: NSObject, MTKViewDelegate {
    private let controller: GameController
    private let commandQueue: MTLCommandQueue
    private let pipelineState: MTLRenderPipelineState

    init(device: MTLDevice, colorPixelFormat: MTLPixelFormat, controller: GameController) throws {
        guard let commandQueue = device.makeCommandQueue() else {
            throw RendererError.missingCommandQueue
        }

        self.controller = controller
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

        controller.stepFrame()
        let viewport = controller.viewport(for: view.bounds.size)
        var vertices = makeSceneVertices(viewport: viewport)

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
        let library = try makeShaderLibrary(device: device)

        let descriptor = MTLRenderPipelineDescriptor()
        descriptor.vertexFunction = library.makeFunction(name: "vertex_main")
        descriptor.fragmentFunction = library.makeFunction(name: "fragment_main")
        descriptor.colorAttachments[0].pixelFormat = pixelFormat
        descriptor.vertexDescriptor = makeVertexDescriptor()

        return try device.makeRenderPipelineState(descriptor: descriptor)
    }

    private static func makeShaderLibrary(device: MTLDevice) throws -> MTLLibrary {
#if SWIFT_PACKAGE
        guard let shaderURL = Bundle.module.url(
            forResource: "AshenShaders",
            withExtension: "metal",
            subdirectory: "Shaders"
        ) else {
            throw RendererError.missingShader
        }

        let shaderSource = try String(contentsOf: shaderURL)
        return try device.makeLibrary(source: shaderSource, options: nil)
#else
        guard let library = device.makeDefaultLibrary() else {
            throw RendererError.missingShader
        }
        return library
#endif
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

    private func makeSceneVertices(viewport: ViewportTransform) -> [Vertex] {
        var vertices: [Vertex] = []
        appendQuad(
            center: viewport.worldToClip(x: 16.5, y: 12.5),
            halfSize: viewport.worldHalfSizeToClip(SIMD2<Float>(0.72, 0.72)),
            color: SIMD4<Float>(0.12, 0.34, 0.42, 1.0),
            to: &vertices
        )

        for unit in controller.units() {
            appendQuad(
                center: viewport.worldToClip(x: unit.x, y: unit.y),
                halfSize: viewport.worldHalfSizeToClip(SIMD2<Float>(0.29, 0.29)),
                color: controller.isSelected(unitID: unit.id)
                    ? SIMD4<Float>(0.95, 0.86, 0.34, 1.0)
                    : SIMD4<Float>(0.76, 0.82, 0.64, 1.0),
                to: &vertices
            )
        }

        for enemy in controller.enemies() {
            appendQuad(
                center: viewport.worldToClip(x: enemy.x, y: enemy.y),
                halfSize: viewport.worldHalfSizeToClip(SIMD2<Float>(0.26, 0.26)),
                color: SIMD4<Float>(0.78, 0.18, 0.16, 1.0),
                to: &vertices
            )
        }

        return vertices
    }
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
