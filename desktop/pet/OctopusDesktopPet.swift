import AppKit
import Foundation

struct AppConfig {
    var statePath = ".octopus/state.json"
    var workerCap = 1
}

struct PetSnapshot: Equatable {
    var state = "heartbeat"
    var workers = 1
    var goal = "No active goal."
    var need = ""
    var source = "octopus"
    var showNeedBubble = false
    var showActionBubbles = false
}

let rows = [
    ".....bbbbb.....",
    "...bbbbbbbbb...",
    "..bbbbbbbbbbb..",
    ".bbbbbbbbbbbbb.",
    ".bbbbebbbebbbb.",
    ".bbbbbbbbbbbbb.",
    "..bbbbbbbbbbb..",
    "...bbbbbbbbb...",
    "..bb.bbb.bb....",
    ".bb..bbb..bb...",
    "bb...b.b...bb..",
    "b....b.b....b..",
]

func parseConfig() -> AppConfig {
    var config = AppConfig()
    var index = 1
    let args = CommandLine.arguments
    while index < args.count {
        switch args[index] {
        case "--state-path":
            index += 1
            if index < args.count { config.statePath = args[index] }
        case "--workers":
            index += 1
            if index < args.count, let count = Int(args[index]) {
                config.workerCap = max(1, min(8, count))
            }
        default:
            break
        }
        index += 1
    }
    return config
}

func colors(for state: String) -> (body: NSColor, head: NSColor, label: String, fallback: String) {
    switch state {
    case "need":
        return (NSColor(calibratedRed: 0.98, green: 0.45, blue: 0.66, alpha: 1), NSColor(calibratedRed: 1.0, green: 0.31, blue: 0.55, alpha: 1), "Need", "🟧")
    case "action":
        return (NSColor(calibratedRed: 0.96, green: 0.62, blue: 0.04, alpha: 1), NSColor(calibratedRed: 0.98, green: 0.45, blue: 0.09, alpha: 1), "Action", "🟨")
    case "feed", "success":
        return (NSColor(calibratedRed: 0.13, green: 0.63, blue: 0.42, alpha: 1), NSColor(calibratedRed: 0.09, green: 0.64, blue: 0.29, alpha: 1), "Feed", "🟩")
    case "blocked", "failed":
        return (NSColor(calibratedRed: 0.72, green: 0.22, blue: 0.15, alpha: 1), NSColor(calibratedRed: 0.86, green: 0.15, blue: 0.15, alpha: 1), "Blocked", "🟥")
    case "memory":
        return (NSColor(calibratedRed: 0.43, green: 0.36, blue: 0.82, alpha: 1), NSColor(calibratedRed: 0.49, green: 0.43, blue: 0.9, alpha: 1), "Memory", "🟪")
    case "harness", "evolution":
        return (NSColor(calibratedRed: 0.15, green: 0.39, blue: 0.92, alpha: 1), NSColor(calibratedRed: 0.31, green: 0.55, blue: 1.0, alpha: 1), "Evolution", "🟦")
    default:
        return (NSColor(calibratedRed: 0.92, green: 0.36, blue: 0.54, alpha: 1), NSColor(calibratedRed: 0.97, green: 0.46, blue: 0.64, alpha: 1), "Heartbeat", "🐙")
    }
}

func loadSnapshot(config: AppConfig) -> PetSnapshot {
    let fallback = PetSnapshot()
    guard let data = FileManager.default.contents(atPath: config.statePath),
          let raw = try? JSONSerialization.jsonObject(with: data),
          let root = raw as? [String: Any] else {
        return fallback
    }

    let goal = dict(root["goal"])
    let goalText = goalDisplay(goal) ?? fallback.goal
    let goalStatus = text(goal?["status"])
    let pendingNeed = latestNeed(root)
    let latestFeed = lastDict(root["feed_traces"])
    let latestVerifier = lastDict(root["field_verifier_results"])
    let latestRun = lastDict(root["parallel_evolution_runs"])
    let lastEvent = dict(root["last_pet_event"])
    let eventFresh = freshEvent(lastEvent)

    var snapshot = fallback
    snapshot.goal = goalText
    snapshot.workers = min(max(workerCount(from: latestRun) ?? 1, 1), config.workerCap)
    snapshot.source = text(lastEvent?["source"])
        ?? latestWorkerId(from: latestRun)
        ?? text(latestFeed?["tentacle"])
        ?? fallback.source
    snapshot.need = pendingNeed
        ?? latestWorkerGoal(from: latestRun)
        ?? text(latestFeed?["need_query"])
        ?? ""
    snapshot.showNeedBubble = pendingNeed != nil

    if goalStatus == "blocked" {
        snapshot.state = "blocked"
    } else if let verifierStatus = text(latestVerifier?["status"]),
              verifierStatus == "failed" || verifierStatus == "unsupported" {
        snapshot.state = "blocked"
    } else if pendingNeed != nil {
        snapshot.state = "need"
    } else if let eventState = text(lastEvent?["state"]), knownState(eventState) {
        snapshot.state = eventState
    } else if latestRun != nil {
        snapshot.state = "harness"
    } else if latestFeed != nil {
        snapshot.state = "feed"
    } else if memoryCount(root) > 0 {
        snapshot.state = "memory"
    }
    snapshot.showActionBubbles = eventFresh && ["action", "harness", "evolution"].contains(snapshot.state)
    return snapshot
}

func knownState(_ value: String) -> Bool {
    ["heartbeat", "need", "action", "feed", "success", "blocked", "failed", "memory", "harness", "evolution"].contains(value)
}

func latestNeed(_ root: [String: Any]) -> String? {
    guard let items = root["need_queue"] as? [[String: Any]] else { return nil }
    for item in items.reversed() {
        guard (text(item["status"]) ?? "pending") == "pending" else { continue }
        if let need = dict(item["need"]), let query = text(need["query"]), !query.isEmpty {
            return query
        }
    }
    return nil
}

func freshEvent(_ event: [String: Any]?) -> Bool {
    guard let timestamp = int(event?["timestamp_secs"]), timestamp > 0 else { return false }
    let age = Date().timeIntervalSince1970 - Double(timestamp)
    return age >= 0 && age <= 8
}

func workerCount(from run: [String: Any]?) -> Int? {
    if let count = int(run?["worker_count"]) { return count }
    return (run?["workers"] as? [Any])?.count
}

func latestWorkerId(from run: [String: Any]?) -> String? {
    guard let workers = run?["workers"] as? [[String: Any]], let first = workers.first else { return nil }
    return text(first["id"])
}

func latestWorkerGoal(from run: [String: Any]?) -> String? {
    guard let workers = run?["workers"] as? [[String: Any]], let first = workers.first else { return nil }
    return text(first["goal"])
}

func memoryCount(_ root: [String: Any]) -> Int {
    guard let memory = dict(root["memory"]), let records = dict(memory["records"]) else { return 0 }
    return records.count
}

func goalDisplay(_ goal: [String: Any]?) -> String? {
    guard let objective = text(goal?["objective"]) else { return nil }
    guard let constraints = goal?["constraints"] as? [String], !constraints.isEmpty else {
        return objective
    }
    let lines = constraints
        .filter { !$0.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty }
        .map { "• \($0)" }
        .joined(separator: "\n")
    return lines.isEmpty ? objective : "\(objective)\n\n\(lines)"
}

func lastDict(_ value: Any?) -> [String: Any]? {
    guard let array = value as? [[String: Any]] else { return nil }
    return array.last
}

func dict(_ value: Any?) -> [String: Any]? {
    value as? [String: Any]
}

func text(_ value: Any?) -> String? {
    if let value = value as? String {
        let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
        return trimmed.isEmpty ? nil : trimmed
    }
    if let value = value as? NSNumber { return value.stringValue }
    return nil
}

func int(_ value: Any?) -> Int? {
    if let value = value as? Int { return value }
    if let value = value as? NSNumber { return value.intValue }
    if let value = value as? String { return Int(value) }
    return nil
}

final class PetView: NSView {
    var snapshot: PetSnapshot
    let workerIndex: Int
    var tick: CGFloat = 0
    var showGoal = false

    init(snapshot: PetSnapshot, workerIndex: Int) {
        self.snapshot = snapshot
        self.workerIndex = workerIndex
        super.init(frame: .zero)
        wantsLayer = true
        layer?.backgroundColor = NSColor.clear.cgColor
        Timer.scheduledTimer(withTimeInterval: 1.0 / 24.0, repeats: true) { [weak self] _ in
            self?.tick += 0.11
            self?.needsDisplay = true
        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override var isOpaque: Bool { false }

    override func mouseDown(with event: NSEvent) {
        showGoal.toggle()
        needsDisplay = true
    }

    func update(snapshot: PetSnapshot) {
        self.snapshot = snapshot
        needsDisplay = true
    }

    override func draw(_ dirtyRect: NSRect) {
        NSColor.clear.setFill()
        dirtyRect.fill()

        let palette = colors(for: snapshot.state)
        let background = NSBezierPath(roundedRect: bounds.insetBy(dx: 8, dy: 8), xRadius: 18, yRadius: 18)
        NSColor(calibratedWhite: 1, alpha: 0.88).setFill()
        background.fill()
        NSColor(calibratedWhite: 0.82, alpha: 0.72).setStroke()
        background.lineWidth = 1
        background.stroke()

        drawGoalPill(palette: palette)
        if snapshot.showNeedBubble, !snapshot.need.isEmpty {
            drawNeedBubble()
        }
        if snapshot.showActionBubbles {
            drawActionBubbles(color: palette.head)
        }
        drawOctopus(palette: palette)
        drawLabel(palette: palette)
        if showGoal {
            drawGoalSheet()
        }
    }

    private func drawGoalPill(palette: (body: NSColor, head: NSColor, label: String, fallback: String)) {
        let text = "Goal"
        let attrs: [NSAttributedString.Key: Any] = [
            .font: NSFont.systemFont(ofSize: 13, weight: .semibold),
            .foregroundColor: NSColor(calibratedWhite: 0.06, alpha: 1),
        ]
        let size = text.size(withAttributes: attrs)
        let rect = NSRect(x: bounds.midX - 42, y: bounds.maxY - 42, width: max(84, size.width + 28), height: 30)
        let path = NSBezierPath(roundedRect: rect, xRadius: 10, yRadius: 10)
        NSColor(calibratedWhite: 1, alpha: 0.94).setFill()
        path.fill()
        palette.head.setStroke()
        path.lineWidth = 1
        path.stroke()
        text.draw(at: NSPoint(x: rect.midX - size.width / 2, y: rect.midY - size.height / 2), withAttributes: attrs)
    }

    private func drawNeedBubble() {
        let text = clipped(snapshot.need, max: 72)
        let rect = NSRect(x: 12, y: bounds.maxY - 126, width: 178, height: 68)
        let path = NSBezierPath(roundedRect: rect, xRadius: 13, yRadius: 13)
        NSColor(calibratedWhite: 1, alpha: 0.95).setFill()
        path.fill()
        NSColor(calibratedWhite: 0.8, alpha: 1).setStroke()
        path.stroke()
        let titleAttrs: [NSAttributedString.Key: Any] = [
            .font: NSFont.systemFont(ofSize: 13, weight: .bold),
            .foregroundColor: NSColor(calibratedWhite: 0.08, alpha: 1),
        ]
        "Need".draw(at: NSPoint(x: rect.minX + 12, y: rect.maxY - 24), withAttributes: titleAttrs)
        let paragraph = NSMutableParagraphStyle()
        paragraph.lineBreakMode = .byWordWrapping
        let attrs: [NSAttributedString.Key: Any] = [
            .font: NSFont.systemFont(ofSize: 11, weight: .regular),
            .foregroundColor: NSColor(calibratedWhite: 0.3, alpha: 1),
            .paragraphStyle: paragraph,
        ]
        text.draw(in: NSRect(x: rect.minX + 12, y: rect.minY + 10, width: rect.width - 24, height: 34), withAttributes: attrs)
    }

    private func drawActionBubbles(color: NSColor) {
        for i in 0..<3 {
            let phase = tick + CGFloat(i) * 0.65
            let radius = CGFloat(6 + i)
            let x = bounds.maxX - CGFloat(58 - i * 22)
            let y = bounds.maxY - 136 + sin(phase) * 12
            color.withAlphaComponent(0.36 + CGFloat(i) * 0.16).setFill()
            NSBezierPath(ovalIn: NSRect(x: x, y: y, width: radius * 2, height: radius * 2)).fill()
        }
    }

    private func drawOctopus(palette: (body: NSColor, head: NSColor, label: String, fallback: String)) {
        let pixel: CGFloat = 12
        let gap: CGFloat = 2
        let width = CGFloat(rows[0].count) * pixel + CGFloat(rows[0].count - 1) * gap
        let height = CGFloat(rows.count) * pixel + CGFloat(rows.count - 1) * gap
        let origin = NSPoint(x: bounds.midX - width / 2, y: bounds.midY - height / 2 - 4 + sin(tick) * 4)

        for (rowIndex, row) in rows.enumerated() {
            for (colIndex, char) in row.enumerated() {
                guard char != "." else { continue }
                var x = origin.x + CGFloat(colIndex) * (pixel + gap)
                var y = origin.y + CGFloat(rows.count - 1 - rowIndex) * (pixel + gap)
                if rowIndex >= 8 {
                    x += sin(tick + CGFloat(colIndex) * 0.5 + CGFloat(workerIndex)) * 4
                    y += cos(tick + CGFloat(colIndex) * 0.4) * 2
                }
                let rect = NSRect(x: x, y: y, width: pixel, height: pixel)
                if char == "e" {
                    NSColor.white.setFill()
                    NSBezierPath(roundedRect: rect, xRadius: 2, yRadius: 2).fill()
                    NSColor(calibratedWhite: 0.06, alpha: 1).setFill()
                    NSBezierPath(roundedRect: rect.insetBy(dx: 4, dy: 4), xRadius: 1, yRadius: 1).fill()
                } else {
                    (rowIndex <= 6 ? palette.head : palette.body).setFill()
                    NSBezierPath(roundedRect: rect, xRadius: 2, yRadius: 2).fill()
                }
            }
        }
    }

    private func drawLabel(palette: (body: NSColor, head: NSColor, label: String, fallback: String)) {
        let suffix = snapshot.workers > 1 ? " \(workerIndex + 1)/\(snapshot.workers)" : ""
        let text = "\(palette.fallback) \(palette.label)\(suffix) · \(snapshot.source)"
        let attrs: [NSAttributedString.Key: Any] = [
            .font: NSFont.systemFont(ofSize: 12, weight: .semibold),
            .foregroundColor: NSColor(calibratedWhite: 0.16, alpha: 1),
        ]
        let clippedText = clipped(text, max: 40)
        let size = clippedText.size(withAttributes: attrs)
        clippedText.draw(at: NSPoint(x: bounds.midX - size.width / 2, y: 20), withAttributes: attrs)
    }

    private func drawGoalSheet() {
        let rect = bounds.insetBy(dx: 18, dy: 28)
        let path = NSBezierPath(roundedRect: rect, xRadius: 16, yRadius: 16)
        NSColor(calibratedWhite: 1, alpha: 0.97).setFill()
        path.fill()
        NSColor(calibratedWhite: 0.78, alpha: 1).setStroke()
        path.stroke()
        let titleAttrs: [NSAttributedString.Key: Any] = [
            .font: NSFont.systemFont(ofSize: 15, weight: .bold),
            .foregroundColor: NSColor(calibratedWhite: 0.06, alpha: 1),
        ]
        "Goal".draw(at: NSPoint(x: rect.minX + 16, y: rect.maxY - 34), withAttributes: titleAttrs)
        let paragraph = NSMutableParagraphStyle()
        paragraph.lineBreakMode = .byWordWrapping
        let attrs: [NSAttributedString.Key: Any] = [
            .font: NSFont.systemFont(ofSize: 12, weight: .regular),
            .foregroundColor: NSColor(calibratedWhite: 0.24, alpha: 1),
            .paragraphStyle: paragraph,
        ]
        clipped(snapshot.goal, max: 260).draw(in: NSRect(x: rect.minX + 16, y: rect.minY + 18, width: rect.width - 32, height: rect.height - 58), withAttributes: attrs)
    }
}

final class PetController {
    let config: AppConfig
    var snapshot: PetSnapshot
    var windows: [NSWindow] = []
    var views: [PetView] = []

    init(config: AppConfig) {
        self.config = config
        self.snapshot = loadSnapshot(config: config)
    }

    func start() {
        rebuildWindows()
        Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { [weak self] _ in
            self?.reload()
        }
    }

    private func reload() {
        let next = loadSnapshot(config: config)
        if next.workers != snapshot.workers {
            snapshot = next
            rebuildWindows()
            return
        }
        snapshot = next
        for view in views {
            view.update(snapshot: next)
        }
    }

    private func rebuildWindows() {
        for window in windows {
            window.orderOut(nil)
            window.close()
        }
        windows.removeAll()
        views.removeAll()

        let screen = NSScreen.main?.visibleFrame ?? NSRect(x: 0, y: 0, width: 1280, height: 800)
        let windowSize = NSSize(width: 280, height: 300)
        let columns = min(snapshot.workers, max(1, Int(screen.width / (windowSize.width + 24))))
        let rowsNeeded = Int(ceil(Double(snapshot.workers) / Double(columns)))
        let startX = screen.midX - CGFloat(columns) * (windowSize.width + 24) / 2
        let startY = screen.midY + CGFloat(rowsNeeded) * (windowSize.height + 24) / 2 - windowSize.height

        for index in 0..<snapshot.workers {
            let col = index % columns
            let row = index / columns
            let frame = NSRect(
                x: startX + CGFloat(col) * (windowSize.width + 24),
                y: startY - CGFloat(row) * (windowSize.height + 24),
                width: windowSize.width,
                height: windowSize.height
            )
            let window = NSWindow(
                contentRect: frame,
                styleMask: [.borderless],
                backing: .buffered,
                defer: false
            )
            let view = PetView(snapshot: snapshot, workerIndex: index)
            window.title = "Octopus Desktop Pet \(index + 1)"
            window.isOpaque = false
            window.backgroundColor = .clear
            window.hasShadow = true
            window.level = .floating
            window.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]
            window.isMovableByWindowBackground = true
            window.contentView = view
            window.makeKeyAndOrderFront(nil)
            windows.append(window)
            views.append(view)
        }
    }
}

func clipped(_ value: String, max: Int) -> String {
    if value.count <= max { return value }
    return String(value.prefix(max)) + "..."
}

let config = parseConfig()
let app = NSApplication.shared
app.setActivationPolicy(.accessory)
let controller = PetController(config: config)
controller.start()
app.activate(ignoringOtherApps: true)
app.run()
