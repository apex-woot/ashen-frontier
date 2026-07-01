import UIKit

struct MapSizeOption {
    let title: String
    let width: UInt16
    let height: UInt16

    var detailText: String {
        "\(width)x\(height)"
    }

    static let compact = MapSizeOption(title: "Compact", width: 256, height: 192)
    static let standard = MapSizeOption(title: "Standard", width: 512, height: 384)
    static let large = MapSizeOption(title: "Large", width: 768, height: 576)
    static let massive = MapSizeOption(title: "Massive", width: 1024, height: 768)
    static let all: [MapSizeOption] = [.compact, .standard, .large, .massive]
}

final class MainMenuViewController: UIViewController {
    private let mapSizeControl = UISegmentedControl(items: MapSizeOption.all.map(\.title))
    private let mapSizeValueLabel = UILabel()
    private var selectedMapSizeIndex = MapSizeOption.all.firstIndex { $0.title == MapSizeOption.standard.title } ?? 0

    init() {
        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override var supportedInterfaceOrientations: UIInterfaceOrientationMask {
        .portrait
    }

    override var preferredInterfaceOrientationForPresentation: UIInterfaceOrientation {
        .portrait
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        navigationController?.setNavigationBarHidden(true, animated: false)
        view.backgroundColor = UIColor(red: 0.035, green: 0.045, blue: 0.04, alpha: 1.0)
        installMenu()
    }

    private func installMenu() {
        let titleLabel = UILabel()
        titleLabel.text = "Ashen Frontier"
        titleLabel.font = .systemFont(ofSize: 42, weight: .bold)
        titleLabel.textColor = UIColor(red: 0.88, green: 0.93, blue: 0.84, alpha: 1.0)
        titleLabel.textAlignment = .center

        let subtitleLabel = UILabel()
        subtitleLabel.text = "Native RTS Prototype"
        subtitleLabel.font = .systemFont(ofSize: 15, weight: .medium)
        subtitleLabel.textColor = UIColor(red: 0.55, green: 0.66, blue: 0.58, alpha: 1.0)
        subtitleLabel.textAlignment = .center

        let mapSizeLabel = UILabel()
        mapSizeLabel.text = "Map Size"
        mapSizeLabel.font = .systemFont(ofSize: 13, weight: .semibold)
        mapSizeLabel.textColor = UIColor(red: 0.72, green: 0.80, blue: 0.68, alpha: 1.0)
        mapSizeLabel.textAlignment = .center

        mapSizeValueLabel.font = .monospacedSystemFont(ofSize: 12, weight: .medium)
        mapSizeValueLabel.textColor = UIColor(red: 0.55, green: 0.66, blue: 0.58, alpha: 1.0)
        mapSizeValueLabel.textAlignment = .center

        mapSizeControl.selectedSegmentIndex = selectedMapSizeIndex
        mapSizeControl.addTarget(self, action: #selector(mapSizeChanged), for: .valueChanged)
        mapSizeControl.selectedSegmentTintColor = UIColor(red: 0.14, green: 0.34, blue: 0.29, alpha: 1.0)
        mapSizeControl.setTitleTextAttributes(
            [.foregroundColor: UIColor(red: 0.91, green: 0.96, blue: 0.88, alpha: 1.0)],
            for: .selected
        )
        mapSizeControl.setTitleTextAttributes(
            [.foregroundColor: UIColor(red: 0.72, green: 0.80, blue: 0.68, alpha: 1.0)],
            for: .normal
        )

        let mapSizeStack = UIStackView(arrangedSubviews: [mapSizeLabel, mapSizeControl, mapSizeValueLabel])
        mapSizeStack.axis = .vertical
        mapSizeStack.spacing = 8
        mapSizeStack.translatesAutoresizingMaskIntoConstraints = false
        updateMapSizeLabel()

        let startButton = makeMenuButton(title: "Start")
        startButton.addTarget(self, action: #selector(startGame), for: .touchUpInside)

        let buttonStack = UIStackView(arrangedSubviews: [startButton])
        buttonStack.axis = .vertical
        buttonStack.spacing = 14
        buttonStack.translatesAutoresizingMaskIntoConstraints = false

        let stack = UIStackView(arrangedSubviews: [titleLabel, subtitleLabel, mapSizeStack, buttonStack])
        stack.axis = .vertical
        stack.alignment = .fill
        stack.spacing = 18
        stack.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(stack)

        NSLayoutConstraint.activate([
            stack.leadingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.leadingAnchor, constant: 28),
            stack.trailingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.trailingAnchor, constant: -28),
            stack.centerYAnchor.constraint(equalTo: view.centerYAnchor),
            mapSizeControl.heightAnchor.constraint(equalToConstant: 36),
            startButton.heightAnchor.constraint(equalToConstant: 54),
        ])
    }

    private func makeMenuButton(title: String) -> UIButton {
        var configuration = UIButton.Configuration.filled()
        configuration.title = title
        configuration.cornerStyle = .medium
        configuration.baseBackgroundColor = UIColor(red: 0.14, green: 0.34, blue: 0.29, alpha: 1.0)
        configuration.baseForegroundColor = UIColor(red: 0.91, green: 0.96, blue: 0.88, alpha: 1.0)

        let button = UIButton(configuration: configuration)
        button.titleLabel?.font = .systemFont(ofSize: 18, weight: .semibold)
        return button
    }

    @objc private func startGame() {
        let gameViewController = GameViewController(mapSize: selectedMapSize)
        navigationController?.pushViewController(gameViewController, animated: true)
    }

    private var selectedMapSize: MapSizeOption {
        MapSizeOption.all[selectedMapSizeIndex]
    }

    @objc private func mapSizeChanged() {
        selectedMapSizeIndex = max(0, mapSizeControl.selectedSegmentIndex)
        updateMapSizeLabel()
    }

    private func updateMapSizeLabel() {
        mapSizeValueLabel.text = selectedMapSize.detailText
    }

}
