import AshenFrontierBridge
import MetalKit
import UIKit

final class GameViewController: UIViewController {
    private static let hudRefreshInterval: TimeInterval = 0.25

    private let mapSize: MapSizeOption
    private var controller: GameController?
    private var renderer: Renderer?
    private var hudTimer: Timer?
    private let hudPanel = UIVisualEffectView(effect: UIBlurEffect(style: .systemUltraThinMaterialDark))
    private let hudStack = UIStackView()
    private let hudHeaderStack = UIStackView()
    private let titleLabel = UILabel()
    private let hudToggleButton = UIButton(type: .system)
    private let fpsValueLabel = UILabel()
    private let tickValueLabel = UILabel()
    private let unitsValueLabel = UILabel()
    private let enemiesValueLabel = UILabel()
    private let selectedValueLabel = UILabel()
    private let commandValueLabel = UILabel()
    private let unitDetailPanel = UIVisualEffectView(effect: UIBlurEffect(style: .systemUltraThinMaterialDark))
    private let unitDetailStack = UIStackView()
    private let unitDetailTitleLabel = UILabel()
    private let unitDetailPositionLabel = UILabel()
    private let unitHealthValueLabel = UILabel()
    private let unitRosterPanel = UIVisualEffectView(effect: UIBlurEffect(style: .systemUltraThinMaterialDark))
    private let unitRosterScrollView = UIScrollView()
    private let unitRosterStack = UIStackView()
    private let commandBar = UIVisualEffectView(effect: UIBlurEffect(style: .systemUltraThinMaterialDark))
    private let commandStack = UIStackView()
    private var commandFeedbackWorkItem: DispatchWorkItem?
    private var isHudMinimized = false
    private var hudExpandedViews: [UIView] = []

    init(mapSize: MapSizeOption = .standard) {
        self.mapSize = mapSize
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

    override func loadView() {
        guard let device = MTLCreateSystemDefaultDevice() else {
            fatalError("Metal is not available on this device")
        }

        let world = RustWorld(width: mapSize.width, height: mapSize.height)
        let controller = GameController(
            world: world,
            controlHint: "Tap unit=select  Tap ground=move  Hold=area select  Drag=pan  Pinch=zoom  Buttons=spawn",
            initialCameraZoom: 2.0
        )
        let gameView = GameView(frame: .zero, device: device)
        gameView.colorPixelFormat = .bgra8Unorm
        gameView.clearColor = MTLClearColor(red: 0.04, green: 0.05, blue: 0.045, alpha: 1.0)
        gameView.enableSetNeedsDisplay = false
        gameView.framebufferOnly = true
        gameView.isOpaque = true
        gameView.presentsWithTransaction = false
        gameView.isPaused = false
        gameView.preferredFramesPerSecond = effectiveFrameRate
        gameView.primaryAction = { [weak self] point, size in
            controller.performPrimaryAction(at: point, in: size)
            self?.refreshHud()
        }
        gameView.selectUnits = { [weak self] point, size in
            controller.selectUnits(near: point, in: size)
            self?.refreshHud()
        }
        gameView.panCamera = { translation, size in
            controller.panCamera(byViewDelta: translation, in: size)
        }
        gameView.zoomCamera = { scale, point, size in
            controller.zoomCamera(by: scale, around: point, in: size)
        }

        do {
            let renderer = try Renderer(device: device, colorPixelFormat: gameView.colorPixelFormat, controller: controller)
            gameView.delegate = renderer
            self.renderer = renderer
        } catch {
            fatalError("failed to create Metal renderer: \(error)")
        }

        self.controller = controller
        view = gameView
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        navigationController?.setNavigationBarHidden(true, animated: false)
        installHud()
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)
        navigationController?.setNavigationBarHidden(true, animated: animated)
        startHudRefreshTimer()
        applyFrameRate()
        refreshHud()
    }

    override func viewWillDisappear(_ animated: Bool) {
        super.viewWillDisappear(animated)
        stopHudRefreshTimer()
    }

    private func startHudRefreshTimer() {
        guard hudTimer == nil else {
            return
        }

        let timer = Timer(
            timeInterval: Self.hudRefreshInterval,
            target: self,
            selector: #selector(refreshHud),
            userInfo: nil,
            repeats: true
        )
        RunLoop.main.add(timer, forMode: .common)
        hudTimer = timer
    }

    private func stopHudRefreshTimer() {
        hudTimer?.invalidate()
        hudTimer = nil
    }

    private func installHud() {
        hudPanel.translatesAutoresizingMaskIntoConstraints = false
        hudPanel.layer.cornerRadius = 10
        hudPanel.layer.cornerCurve = .continuous
        hudPanel.clipsToBounds = true
        hudPanel.layer.borderWidth = 1
        hudPanel.layer.borderColor = UIColor(red: 0.70, green: 0.78, blue: 0.66, alpha: 0.20).cgColor
        view.addSubview(hudPanel)

        hudStack.axis = .vertical
        hudStack.spacing = 7
        hudStack.translatesAutoresizingMaskIntoConstraints = false
        hudPanel.contentView.addSubview(hudStack)

        hudHeaderStack.axis = .horizontal
        hudHeaderStack.alignment = .center
        hudHeaderStack.spacing = 8
        hudStack.addArrangedSubview(hudHeaderStack)

        titleLabel.text = "ASHEN FRONTIER"
        titleLabel.font = .systemFont(ofSize: 11, weight: .bold)
        titleLabel.textColor = UIColor(red: 0.92, green: 0.96, blue: 0.86, alpha: 1.0)
        titleLabel.setContentHuggingPriority(.defaultLow, for: .horizontal)
        titleLabel.setContentCompressionResistancePriority(.required, for: .vertical)
        hudHeaderStack.addArrangedSubview(titleLabel)

        configureHudToggleButton()
        hudHeaderStack.addArrangedSubview(hudToggleButton)

        let topDivider = makeDivider()
        let fpsRow = makeMetricRow(title: "FPS", valueLabel: fpsValueLabel)
        let tickRow = makeMetricRow(title: "TICK", valueLabel: tickValueLabel)
        let unitsRow = makeMetricRow(title: "UNITS", valueLabel: unitsValueLabel)
        let enemiesRow = makeMetricRow(title: "ENEMY", valueLabel: enemiesValueLabel)
        let selectedRow = makeMetricRow(title: "SELECTED", valueLabel: selectedValueLabel)
        let commandRow = makeMetricRow(title: "COMMAND", valueLabel: commandValueLabel)

        [topDivider, fpsRow, tickRow, unitsRow, enemiesRow, selectedRow, commandRow].forEach {
            hudStack.addArrangedSubview($0)
        }
        hudExpandedViews = [tickRow, unitsRow, enemiesRow, selectedRow, commandRow]

        installUnitDetailPanel()

        unitRosterPanel.translatesAutoresizingMaskIntoConstraints = false
        unitRosterPanel.layer.cornerRadius = 10
        unitRosterPanel.layer.cornerCurve = .continuous
        unitRosterPanel.clipsToBounds = true
        unitRosterPanel.layer.borderWidth = 1
        unitRosterPanel.layer.borderColor = UIColor(red: 0.70, green: 0.78, blue: 0.66, alpha: 0.16).cgColor
        view.addSubview(unitRosterPanel)

        unitRosterScrollView.translatesAutoresizingMaskIntoConstraints = false
        unitRosterScrollView.showsHorizontalScrollIndicator = false
        unitRosterScrollView.alwaysBounceHorizontal = true
        unitRosterScrollView.delaysContentTouches = false
        unitRosterPanel.contentView.addSubview(unitRosterScrollView)

        unitRosterStack.axis = .horizontal
        unitRosterStack.alignment = .center
        unitRosterStack.spacing = 8
        unitRosterStack.translatesAutoresizingMaskIntoConstraints = false
        unitRosterScrollView.addSubview(unitRosterStack)

        commandBar.translatesAutoresizingMaskIntoConstraints = false
        commandBar.layer.cornerRadius = 10
        commandBar.layer.cornerCurve = .continuous
        commandBar.clipsToBounds = true
        commandBar.layer.borderWidth = 1
        commandBar.layer.borderColor = UIColor(red: 0.70, green: 0.78, blue: 0.66, alpha: 0.16).cgColor
        view.addSubview(commandBar)

        commandStack.axis = .horizontal
        commandStack.alignment = .center
        commandStack.distribution = .fillEqually
        commandStack.spacing = 8
        commandStack.translatesAutoresizingMaskIntoConstraints = false
        commandBar.contentView.addSubview(commandStack)

        commandStack.addArrangedSubview(makeCommandButton(title: "All", action: #selector(selectAllUnits(_:))))
        commandStack.addArrangedSubview(makeCommandButton(title: "Clear", action: #selector(clearSelection(_:))))
        commandStack.addArrangedSubview(makeCommandButton(title: "Wrk", action: #selector(spawnWorker(_:))))
        commandStack.addArrangedSubview(makeCommandButton(title: "Rng", action: #selector(spawnRanger(_:))))
        commandStack.addArrangedSubview(makeCommandButton(title: "Sld", action: #selector(spawnSoldier(_:))))
        commandStack.addArrangedSubview(makeCommandButton(title: "Horde", action: #selector(spawnHorde(_:))))

        NSLayoutConstraint.activate([
            hudPanel.leadingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.leadingAnchor, constant: 12),
            hudPanel.topAnchor.constraint(equalTo: view.safeAreaLayoutGuide.topAnchor, constant: 10),
            hudPanel.widthAnchor.constraint(equalToConstant: 188),

            hudStack.leadingAnchor.constraint(equalTo: hudPanel.contentView.leadingAnchor, constant: 12),
            hudStack.trailingAnchor.constraint(equalTo: hudPanel.contentView.trailingAnchor, constant: -12),
            hudStack.topAnchor.constraint(equalTo: hudPanel.contentView.topAnchor, constant: 12),
            hudStack.bottomAnchor.constraint(equalTo: hudPanel.contentView.bottomAnchor, constant: -12),

            unitDetailPanel.leadingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.leadingAnchor, constant: 12),
            unitDetailPanel.trailingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.trailingAnchor, constant: -12),
            unitDetailPanel.bottomAnchor.constraint(equalTo: unitRosterPanel.topAnchor, constant: -10),
            unitDetailPanel.heightAnchor.constraint(equalToConstant: 78),

            unitRosterPanel.leadingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.leadingAnchor, constant: 12),
            unitRosterPanel.trailingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.trailingAnchor, constant: -12),
            unitRosterPanel.bottomAnchor.constraint(equalTo: commandBar.topAnchor, constant: -10),
            unitRosterPanel.heightAnchor.constraint(equalToConstant: 76),

            unitRosterScrollView.leadingAnchor.constraint(equalTo: unitRosterPanel.contentView.leadingAnchor, constant: 10),
            unitRosterScrollView.trailingAnchor.constraint(equalTo: unitRosterPanel.contentView.trailingAnchor, constant: -10),
            unitRosterScrollView.topAnchor.constraint(equalTo: unitRosterPanel.contentView.topAnchor, constant: 10),
            unitRosterScrollView.bottomAnchor.constraint(equalTo: unitRosterPanel.contentView.bottomAnchor, constant: -10),

            unitRosterStack.leadingAnchor.constraint(equalTo: unitRosterScrollView.contentLayoutGuide.leadingAnchor),
            unitRosterStack.trailingAnchor.constraint(equalTo: unitRosterScrollView.contentLayoutGuide.trailingAnchor),
            unitRosterStack.topAnchor.constraint(equalTo: unitRosterScrollView.contentLayoutGuide.topAnchor),
            unitRosterStack.bottomAnchor.constraint(equalTo: unitRosterScrollView.contentLayoutGuide.bottomAnchor),
            unitRosterStack.heightAnchor.constraint(equalTo: unitRosterScrollView.frameLayoutGuide.heightAnchor),

            commandBar.leadingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.leadingAnchor, constant: 12),
            commandBar.trailingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.trailingAnchor, constant: -12),
            commandBar.bottomAnchor.constraint(equalTo: view.safeAreaLayoutGuide.bottomAnchor, constant: -12),
            commandBar.heightAnchor.constraint(equalToConstant: 42),

            commandStack.leadingAnchor.constraint(equalTo: commandBar.contentView.leadingAnchor, constant: 10),
            commandStack.trailingAnchor.constraint(equalTo: commandBar.contentView.trailingAnchor, constant: -10),
            commandStack.centerYAnchor.constraint(equalTo: commandBar.contentView.centerYAnchor),
        ])
    }

    private func installUnitDetailPanel() {
        unitDetailPanel.translatesAutoresizingMaskIntoConstraints = false
        unitDetailPanel.layer.cornerRadius = 10
        unitDetailPanel.layer.cornerCurve = .continuous
        unitDetailPanel.clipsToBounds = true
        unitDetailPanel.layer.borderWidth = 1
        unitDetailPanel.layer.borderColor = UIColor(red: 0.70, green: 0.78, blue: 0.66, alpha: 0.16).cgColor
        view.addSubview(unitDetailPanel)

        unitDetailStack.axis = .vertical
        unitDetailStack.spacing = 7
        unitDetailStack.translatesAutoresizingMaskIntoConstraints = false
        unitDetailPanel.contentView.addSubview(unitDetailStack)

        unitDetailTitleLabel.font = .systemFont(ofSize: 12, weight: .bold)
        unitDetailTitleLabel.textColor = UIColor(red: 0.92, green: 0.96, blue: 0.86, alpha: 1.0)
        unitDetailStack.addArrangedSubview(unitDetailTitleLabel)

        unitDetailPositionLabel.font = .monospacedSystemFont(ofSize: 10, weight: .medium)
        unitDetailPositionLabel.textColor = UIColor(red: 0.67, green: 0.75, blue: 0.63, alpha: 1.0)
        unitDetailStack.addArrangedSubview(unitDetailPositionLabel)

        unitDetailStack.addArrangedSubview(makeMetricRow(title: "HP", valueLabel: unitHealthValueLabel))

        NSLayoutConstraint.activate([
            unitDetailStack.leadingAnchor.constraint(equalTo: unitDetailPanel.contentView.leadingAnchor, constant: 12),
            unitDetailStack.trailingAnchor.constraint(equalTo: unitDetailPanel.contentView.trailingAnchor, constant: -12),
            unitDetailStack.topAnchor.constraint(equalTo: unitDetailPanel.contentView.topAnchor, constant: 10),
            unitDetailStack.bottomAnchor.constraint(lessThanOrEqualTo: unitDetailPanel.contentView.bottomAnchor, constant: -10),
        ])
    }

    private func configureHudToggleButton() {
        var configuration = UIButton.Configuration.plain()
        configuration.image = UIImage(systemName: "chevron.up")
        configuration.baseForegroundColor = UIColor(red: 0.88, green: 0.93, blue: 0.84, alpha: 1.0)
        configuration.contentInsets = NSDirectionalEdgeInsets(top: 2, leading: 4, bottom: 2, trailing: 4)

        hudToggleButton.configuration = configuration
        hudToggleButton.accessibilityLabel = "Minimize HUD"
        hudToggleButton.setContentHuggingPriority(.required, for: .horizontal)
        hudToggleButton.addTarget(self, action: #selector(toggleHudMinimized), for: .touchDown)
    }

    private func makeMetricRow(title: String, valueLabel: UILabel) -> UIStackView {
        let titleLabel = UILabel()
        titleLabel.text = title
        titleLabel.font = .monospacedSystemFont(ofSize: 10, weight: .semibold)
        titleLabel.textColor = UIColor(red: 0.55, green: 0.66, blue: 0.58, alpha: 1.0)

        valueLabel.font = .monospacedSystemFont(ofSize: 12, weight: .semibold)
        valueLabel.textColor = UIColor(red: 0.88, green: 0.93, blue: 0.84, alpha: 1.0)
        valueLabel.textAlignment = .right
        valueLabel.adjustsFontSizeToFitWidth = true
        valueLabel.minimumScaleFactor = 0.72
        valueLabel.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)

        let row = UIStackView(arrangedSubviews: [titleLabel, valueLabel])
        row.axis = .horizontal
        row.alignment = .firstBaseline
        row.spacing = 8
        return row
    }

    private func makeDivider() -> UIView {
        let divider = UIView()
        divider.backgroundColor = UIColor(red: 0.70, green: 0.78, blue: 0.66, alpha: 0.16)
        divider.translatesAutoresizingMaskIntoConstraints = false
        divider.heightAnchor.constraint(equalToConstant: 1).isActive = true
        return divider
    }

    private func makeCommandButton(title: String, action: Selector) -> UIButton {
        var configuration = UIButton.Configuration.filled()
        configuration.title = title.uppercased()
        configuration.baseBackgroundColor = commandButtonColor()
        configuration.baseForegroundColor = UIColor(red: 0.88, green: 0.93, blue: 0.84, alpha: 1.0)
        configuration.cornerStyle = .medium
        configuration.contentInsets = NSDirectionalEdgeInsets(top: 7, leading: 8, bottom: 7, trailing: 8)

        let button = UIButton(type: .system)
        button.configuration = configuration
        button.titleLabel?.font = .systemFont(ofSize: 11, weight: .bold)
        button.configurationUpdateHandler = { button in
            var updatedConfiguration = button.configuration
            updatedConfiguration?.baseBackgroundColor = self.commandButtonColor(
                isHighlighted: button.isHighlighted,
                isSelected: button.isSelected
            )
            updatedConfiguration?.baseForegroundColor = button.isHighlighted
                ? UIColor(red: 1.00, green: 0.96, blue: 0.74, alpha: 1.0)
                : UIColor(red: 0.88, green: 0.93, blue: 0.84, alpha: 1.0)
            button.configuration = updatedConfiguration
        }
        button.addTarget(self, action: action, for: .touchDown)
        return button
    }

    private func makeUnitButton(for unit: AfEntityPosition, isSelected: Bool) -> UIButton {
        var configuration = UIButton.Configuration.filled()
        configuration.title = "U\(unit.id)"
        configuration.subtitle = "\(Int(unit.x)), \(Int(unit.y))"
        configuration.baseBackgroundColor = unitButtonColor(isSelected: isSelected)
        configuration.baseForegroundColor = UIColor(red: 0.92, green: 0.96, blue: 0.86, alpha: 1.0)
        configuration.cornerStyle = .medium
        configuration.contentInsets = NSDirectionalEdgeInsets(top: 6, leading: 10, bottom: 6, trailing: 10)

        let button = UIButton(type: .system)
        button.configuration = configuration
        button.tag = Int(unit.id)
        button.isSelected = isSelected
        button.titleLabel?.font = .monospacedSystemFont(ofSize: 12, weight: .bold)
        button.accessibilityLabel = "Select unit \(unit.id)"
        button.configurationUpdateHandler = { button in
            var updatedConfiguration = button.configuration
            updatedConfiguration?.baseBackgroundColor = self.unitButtonColor(
                isHighlighted: button.isHighlighted,
                isSelected: button.isSelected
            )
            updatedConfiguration?.baseForegroundColor = button.isHighlighted
                ? UIColor(red: 1.00, green: 0.98, blue: 0.80, alpha: 1.0)
                : UIColor(red: 0.92, green: 0.96, blue: 0.86, alpha: 1.0)
            button.configuration = updatedConfiguration
        }
        button.addTarget(self, action: #selector(selectRosterUnit(_:)), for: .touchDown)
        button.widthAnchor.constraint(greaterThanOrEqualToConstant: 68).isActive = true
        return button
    }

    private func commandButtonColor(isHighlighted: Bool = false, isSelected: Bool = false) -> UIColor {
        if isHighlighted {
            return UIColor(red: 0.36, green: 0.30, blue: 0.13, alpha: 1.0)
        }

        if isSelected {
            return UIColor(red: 0.70, green: 0.56, blue: 0.18, alpha: 0.98)
        }

        return UIColor(red: 0.13, green: 0.20, blue: 0.15, alpha: 0.92)
    }

    private func unitButtonColor(isHighlighted: Bool = false, isSelected: Bool = false) -> UIColor {
        if isHighlighted {
            return UIColor(red: 0.36, green: 0.30, blue: 0.13, alpha: 1.0)
        }

        if isSelected {
            return UIColor(red: 0.70, green: 0.56, blue: 0.18, alpha: 0.96)
        }

        return UIColor(red: 0.11, green: 0.17, blue: 0.13, alpha: 0.92)
    }

    private func showCommandFeedback(on button: UIButton) {
        UIImpactFeedbackGenerator(style: .light).impactOccurred()
        commandFeedbackWorkItem?.cancel()

        button.isSelected = true
        button.setNeedsUpdateConfiguration()

        let workItem = DispatchWorkItem { [weak button] in
            button?.isSelected = false
            button?.setNeedsUpdateConfiguration()
        }
        commandFeedbackWorkItem = workItem
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.35, execute: workItem)
    }

    private func showSelectionFeedback() {
        UISelectionFeedbackGenerator().selectionChanged()
    }

    private func updateHudMinimizedState() {
        hudExpandedViews.forEach { $0.isHidden = isHudMinimized }

        var configuration = hudToggleButton.configuration
        configuration?.image = UIImage(systemName: isHudMinimized ? "chevron.down" : "chevron.up")
        hudToggleButton.configuration = configuration
        hudToggleButton.accessibilityLabel = isHudMinimized ? "Expand HUD" : "Minimize HUD"
    }

    private func applyFrameRate() {
        let preferredFrameRate = effectiveFrameRate
        let metalView = view as? MTKView
        metalView?.preferredFramesPerSecond = preferredFrameRate
        metalView?.isPaused = false
    }

    private var effectiveFrameRate: Int {
        UIScreen.main.maximumFramesPerSecond
    }

    @objc private func refreshHud() {
        fpsValueLabel.text = String(format: "%.1f", controller?.fps ?? 0)
        tickValueLabel.text = controller.map { "\($0.world.tick)" } ?? "--"
        unitsValueLabel.text = controller.map { "\($0.world.unitCount)" } ?? "--"
        enemiesValueLabel.text = controller.map { "\($0.world.enemyCount)" } ?? "--"
        selectedValueLabel.text = controller.map { "\($0.selectedUnitIDs.count)" } ?? "--"
        commandValueLabel.text = controller.map { commandStatusLabel($0.lastCommandStatus) } ?? "--"
        refreshUnitDetails()
        refreshUnitRoster()
    }

    private func refreshUnitDetails() {
        if let details = controller?.focusedUnitDetails() {
            unitDetailPanel.isHidden = false
            unitDetailTitleLabel.text = "\(details.kind.uppercased()) U\(details.id)"
            unitDetailPositionLabel.text = "POS \(format(details.position.x)), \(format(details.position.y))"
            unitHealthValueLabel.text = format(details.health)
            return
        }

        guard let details = controller?.focusedEnemyDetails() else {
            unitDetailPanel.isHidden = true
            return
        }

        unitDetailPanel.isHidden = false
        unitDetailTitleLabel.text = "\(details.kind.uppercased()) E\(details.id)"
        unitDetailPositionLabel.text = "POS \(format(details.position.x)), \(format(details.position.y))"
        unitHealthValueLabel.text = format(details.health)
    }

    private func refreshUnitRoster() {
        guard let controller else {
            return
        }

        unitRosterStack.arrangedSubviews.forEach { view in
            unitRosterStack.removeArrangedSubview(view)
            view.removeFromSuperview()
        }

        let units = controller.units().sorted { $0.id < $1.id }
        if units.isEmpty {
            let emptyLabel = UILabel()
            emptyLabel.text = "NO UNITS"
            emptyLabel.font = .systemFont(ofSize: 11, weight: .bold)
            emptyLabel.textColor = UIColor(red: 0.55, green: 0.66, blue: 0.58, alpha: 1.0)
            unitRosterStack.addArrangedSubview(emptyLabel)
            return
        }

        for unit in units {
            unitRosterStack.addArrangedSubview(
                makeUnitButton(for: unit, isSelected: controller.selectedUnitIDs.contains(unit.id))
            )
        }
    }

    private func format(_ value: Float) -> String {
        if value.rounded() == value {
            return String(format: "%.0f", value)
        }

        return String(format: "%.1f", value)
    }

    @objc private func toggleHudMinimized() {
        showSelectionFeedback()
        isHudMinimized.toggle()
        UIView.animate(withDuration: 0.18) {
            self.updateHudMinimizedState()
            self.view.layoutIfNeeded()
        }
    }

    @objc private func selectRosterUnit(_ sender: UIButton) {
        showSelectionFeedback()
        controller?.selectUnit(id: UInt32(sender.tag))
        refreshHud()
    }

    @objc private func selectAllUnits(_ sender: UIButton) {
        showCommandFeedback(on: sender)
        controller?.selectAllUnits()
        refreshHud()
    }

    @objc private func clearSelection(_ sender: UIButton) {
        showCommandFeedback(on: sender)
        controller?.clearSelection()
        refreshHud()
    }

    @objc private func spawnHorde(_ sender: UIButton) {
        showCommandFeedback(on: sender)
        controller?.spawnHorde()
        refreshHud()
    }

    @objc private func spawnWorker(_ sender: UIButton) {
        showCommandFeedback(on: sender)
        controller?.spawnUnit(kind: .worker)
        refreshHud()
    }

    @objc private func spawnRanger(_ sender: UIButton) {
        showCommandFeedback(on: sender)
        controller?.spawnUnit(kind: .ranger)
        refreshHud()
    }

    @objc private func spawnSoldier(_ sender: UIButton) {
        showCommandFeedback(on: sender)
        controller?.spawnUnit(kind: .soldier)
        refreshHud()
    }
}
