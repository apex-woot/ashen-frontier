import Foundation

@MainActor final class AppSettings {
    static let shared = AppSettings()

    private let defaults: UserDefaults
    private let frameRateKey = "ashen_frontier_target_frame_rate"

    init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
    }

    var targetFrameRate: Int {
        get {
            let storedValue = defaults.integer(forKey: frameRateKey)
            return Self.supportedFrameRates.contains(storedValue) ? storedValue : 120
        }
        set {
            defaults.set(Self.supportedFrameRates.contains(newValue) ? newValue : 120, forKey: frameRateKey)
        }
    }

    static let supportedFrameRates = [60, 120]
}
