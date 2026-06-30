import AppKit
import Darwin
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
    var fieldPool = ""
    var workerNeeds: [String] = []
    var workerSources: [String] = []
    var workerStates: [String] = []
    var workerUpdatedAt: [Int] = []
    var showNeedBubble = false
    var showActionBubbles = false
}

struct FieldPoolObservation {
    var summary = ""
    var sources: [String] = []
    var states: [String] = []
    var updatedAt: [Int] = []
}

let peerFieldIds = ["math", "search", "code", "swe", "research", "computer-use", "ib", "robotics", "write", "translate"]

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

let observationFreshSeconds: TimeInterval = 8
let eventStateFreshSeconds: TimeInterval = 300
let workerPolicyDefault = "workers are execution slots from the peer field pool; fields stay peer"

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
            guard index < args.count, let count = Int(args[index]), (1...8).contains(count) else {
                usageError("desktop pet workers must be between 1 and 8")
            }
            config.workerCap = count
        default:
            break
        }
        index += 1
    }
    return config
}

func usageError(_ message: String) -> Never {
    FileHandle.standardError.write(Data((message + "\n").utf8))
    Darwin.exit(64)
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
    let fallback = fallbackSnapshot(config: config)
    guard let data = FileManager.default.contents(atPath: config.statePath),
          let raw = try? JSONSerialization.jsonObject(with: data),
          let root = raw as? [String: Any] else {
        return fallback
    }

    let goal = dict(root["goal"])
    let goalText = goalDisplay(goal) ?? fallback.goal
    let goalStatus = text(goal?["status"])?.lowercased()
    let pendingNeed = latestNeed(root)
    let latestFeed = lastDict(root["feed_traces"])
    let latestVerifier = lastDict(root["field_verifier_results"])
    let latestRun = lastDict(root["parallel_evolution_runs"])
    let fieldPool = observeFieldPool(root)
    let lastEvent = dict(root["last_pet_event"])
    let eventFresh = freshEvent(lastEvent, maxAge: eventStateFreshSeconds)
    let eventBubbleFresh = freshEvent(lastEvent)
    let eventState = text(lastEvent?["state"])
    let freshNeedEvent = eventBubbleFresh && eventState == "need"
    let eventSummary = text(lastEvent?["summary"])
    let runWorkerNeeds = latestWorkerNeeds(from: latestRun, root: root)
    let runWorkerSources = latestWorkerSources(from: latestRun)
    let runWorkerStates = latestWorkerStates(from: latestRun)
    let runWorkerUpdatedAt = latestWorkerUpdatedAt(from: latestRun)
    let runHasActiveWorker = firstNonEmpty(runWorkerNeeds) != nil
        || runWorkerUpdatedAt.contains { freshTimestamp($0) }
    let activeRunWorkerCount = runHasActiveWorker ? workerCount(from: latestRun) : nil
    let workerNeeds = runHasActiveWorker ? runWorkerNeeds : Array(repeating: "", count: fieldPool.sources.count)
    let workerSources = runHasActiveWorker ? runWorkerSources : fieldPool.sources
    let workerStates = runHasActiveWorker ? runWorkerStates : fieldPool.states
    let workerUpdatedAt = runHasActiveWorker ? runWorkerUpdatedAt : fieldPool.updatedAt

    var snapshot = fallback
    snapshot.goal = goalText
    snapshot.fieldPool = fieldPool.summary
    snapshot.workers = observerWindowCount(runWorkers: activeRunWorkerCount, config: config)
    snapshot.workerNeeds = workerNeeds
    snapshot.workerSources = workerSources
    snapshot.workerStates = workerStates
    snapshot.workerUpdatedAt = workerUpdatedAt
    snapshot.source = (eventFresh ? text(lastEvent?["source"]) : nil)
        ?? firstNonEmpty(workerSources)
        ?? (runHasActiveWorker ? latestWorkerId(from: latestRun) : nil)
        ?? text(latestFeed?["tentacle"])
        ?? fallback.source
    snapshot.need = pendingNeed
        ?? firstNonEmpty(workerNeeds)
        ?? (freshNeedEvent ? eventSummary : nil)
        ?? text(latestFeed?["need_query"])
        ?? ""
    snapshot.showNeedBubble = pendingNeed != nil || firstNonEmpty(workerNeeds) != nil || (freshNeedEvent && eventSummary != nil)

    if goalStatus == "blocked" {
        snapshot.state = "blocked"
    } else if let verifierStatus = text(latestVerifier?["status"]),
              verifierStatus == "failed" || verifierStatus == "unsupported" {
        snapshot.state = "blocked"
    } else if pendingNeed != nil {
        snapshot.state = "need"
    } else if eventFresh, let eventState = eventState, knownState(eventState) {
        snapshot.state = eventState
    } else if runHasActiveWorker {
        snapshot.state = "harness"
    } else if latestFeed != nil {
        snapshot.state = "feed"
    } else if memoryCount(root) > 0 {
        snapshot.state = "memory"
    }
    snapshot.showActionBubbles = eventBubbleFresh
        && (eventState.map {
            ["action", "harness", "evolution", "feed", "success"].contains($0)
        } ?? false)
    return snapshot
}

func fallbackSnapshot(config: AppConfig) -> PetSnapshot {
    var snapshot = PetSnapshot()
    snapshot.workers = observerWindowCount(runWorkers: nil, config: config)
    return snapshot
}

func observerWindowCount(runWorkers: Int?, config: AppConfig) -> Int {
    let requested = max(1, min(8, config.workerCap))
    let active = runWorkers.map { max(1, min(8, $0)) } ?? requested
    return max(requested, active)
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

func freshEvent(_ event: [String: Any]?, maxAge: TimeInterval = observationFreshSeconds) -> Bool {
    guard let timestamp = int(event?["timestamp_secs"]), timestamp > 0 else { return false }
    return freshTimestamp(timestamp, maxAge: maxAge)
}

func freshTimestamp(_ timestamp: Int, maxAge: TimeInterval = observationFreshSeconds) -> Bool {
    guard timestamp > 0 else { return false }
    let age = Date().timeIntervalSince1970 - Double(timestamp)
    return age >= 0 && age <= maxAge
}

func workerCount(from run: [String: Any]?) -> Int? {
    if let count = int(run?["worker_count"]) { return count }
    return (run?["workers"] as? [Any])?.count
}

func latestWorkerId(from run: [String: Any]?) -> String? {
    guard let workers = run?["workers"] as? [[String: Any]], let first = workers.first else { return nil }
    return text(first["id"])
}

func latestWorkerNeeds(from run: [String: Any]?, root: [String: Any]) -> [String] {
    guard let workers = run?["workers"] as? [[String: Any]], !workers.isEmpty else { return [] }
    let queue = pendingQueuedNeedQueries(root)
    return workers.map { worker in
        if let index = int(worker["queued_need_index"]), let query = queue[index] {
            return query
        }
        if freshTimestamp(int(worker["updated_at_secs"]) ?? 0) {
            return workerNeedLabel(worker)
        }
        return ""
    }
}

func workerNeedLabel(_ worker: [String: Any]) -> String {
    if let field = text(worker["field"]) {
        if let task = text(worker["mini_task"]) {
            return "\(field) · \(task)"
        }
        return "\(field) · peer field"
    }
    return ""
}

func latestWorkerSources(from run: [String: Any]?) -> [String] {
    guard let workers = run?["workers"] as? [[String: Any]], !workers.isEmpty else { return [] }
    return workers.map { worker in
        if let field = text(worker["field"]) {
            if let task = text(worker["mini_task"]) {
                return "field:\(field)/\(task)"
            }
            return "field:\(field)"
        }
        return text(worker["id"]) ?? "worker"
    }
}

func latestWorkerStates(from run: [String: Any]?) -> [String] {
    guard let workers = run?["workers"] as? [[String: Any]], !workers.isEmpty else { return [] }
    return workers.map { worker in
        switch text(worker["status"])?.lowercased() {
        case "satisfied":
            return "feed"
        case "failed", "unsupported":
            return "blocked"
        case "partial":
            return "harness"
        default:
            return ""
        }
    }
}

func latestWorkerUpdatedAt(from run: [String: Any]?) -> [Int] {
    guard let workers = run?["workers"] as? [[String: Any]], !workers.isEmpty else { return [] }
    return workers.map { worker in int(worker["updated_at_secs"]) ?? 0 }
}

func observeFieldPool(_ root: [String: Any]) -> FieldPoolObservation {
    if let pool = dict(root["field_pool"]),
       let slots = pool["slots"] as? [[String: Any]],
       !slots.isEmpty {
        return observeSerializedFieldPool(pool, slots: slots, root: root)
    }
    return observeLegacyFieldPool(root)
}

func observeSerializedFieldPool(_ pool: [String: Any], slots: [[String: Any]], root: [String: Any]) -> FieldPoolObservation {
    let activeField = text(pool["active_slot_field"])
    let orderedSlots = activeField
        .map { active in slots.sorted { slotPriority($0, active: active) < slotPriority($1, active: active) } }
        ?? slots
    var sources: [String] = []
    var states: [String] = []
    var updatedAt: [Int] = []

    for slot in orderedSlots {
        guard let field = text(slot["field"]) else { continue }
        let task = text(slot["latest_mini_task"]) ?? text(slot["next_mini_task"])
        let taskSuffix = task.map { "/\($0)" } ?? ""
        let worker = text(slot["latest_worker_id"]).map { "\($0):" } ?? "pool:"
        sources.append("\(worker)\(field)\(taskSuffix)")
        states.append(workerState(from: text(slot["latest_status"]) ?? text(slot["latest_worker_status"])))
        updatedAt.append(int(slot["latest_updated_at_secs"]) ?? 0)
    }

    let count = int(pool["field_slot_count"]) ?? int(pool["field_count"]) ?? slots.count
    let workers = int(pool["latest_worker_slot_count"]) ?? 0
    let completed = int(pool["completed_fields"]) ?? slots.filter { bool($0["completed"]) == true }.count
    let policy = text(pool["worker_slots"]) ?? text(pool["policy"]) ?? workerPolicyDefault
    let reason = text(pool["active_slot_reason"]).map { "\nreason: \($0)" } ?? ""
    let latestRun = lastDict(root["parallel_evolution_runs"])
    let runDetail = parallelRunPoolDetail(from: latestRun, activeWorkers: workers)
    return FieldPoolObservation(
        summary: "\(count) peer fields · \(workers) worker slots · \(completed) complete · active \(activeField ?? "none")\(reason)\n\(policy)\(runDetail)",
        sources: sources,
        states: states,
        updatedAt: updatedAt
    )
}

func observeLegacyFieldPool(_ root: [String: Any]) -> FieldPoolObservation {
    let verifierByField = latestVerifierByField(root)
    let workerByField = latestWorkerByField(root)
    let latestRun = lastDict(root["parallel_evolution_runs"])
    let policy = text(latestRun?["worker_policy"]) ?? workerPolicyDefault
    let activeField = peerFieldIds.first { field in
        let status = text(workerByField[field]?["status"]) ?? text(verifierByField[field]?["status"])
        return status?.lowercased() != "satisfied"
    }
    let orderedFields = activeField
        .map { active in [active] + peerFieldIds.filter { field in field != active } }
        ?? peerFieldIds
    var sources: [String] = []
    var states: [String] = []
    var updatedAt: [Int] = []
    var completed = 0

    for field in orderedFields {
        let worker = workerByField[field]
        let verifier = verifierByField[field]
        let status = text(worker?["status"]) ?? text(verifier?["status"])
        let task = text(worker?["mini_task"]) ?? latestMiniTaskForField(root, field: field)
        if status?.lowercased() == "satisfied" {
            completed += 1
        }
        let taskSuffix = task.map { "/\($0)" } ?? ""
        sources.append("pool:\(field)\(taskSuffix)")
        states.append(workerState(from: status))
        updatedAt.append(int(worker?["updated_at_secs"]) ?? 0)
    }

    let activeWorkers = workerCount(from: latestRun) ?? 0
    let runDetail = parallelRunPoolDetail(from: latestRun, activeWorkers: activeWorkers)
    return FieldPoolObservation(
        summary: "\(peerFieldIds.count) peer fields · \(completed) complete · active \(activeField ?? "none")\n\(policy)\(runDetail)",
        sources: sources,
        states: states,
        updatedAt: updatedAt
    )
}

func parallelRunPoolDetail(from run: [String: Any]?, activeWorkers: Int) -> String {
    guard let run else { return "" }
    let requested = int(run["requested_worker_count"]) ?? int(run["worker_count"]) ?? activeWorkers
    let active = int(run["worker_count"]) ?? activeWorkers
    let candidates = stringArray(run["candidate_fields"])
    let candidateText = candidates.isEmpty ? "all peer fields" : candidates.joined(separator: ", ")
    return "\nrequested \(requested) · active \(active) · candidates \(candidateText)"
}

func slotPriority(_ slot: [String: Any], active: String) -> Int {
    text(slot["field"]) == active ? 0 : 1
}

func latestWorkerByField(_ root: [String: Any]) -> [String: [String: Any]] {
    guard let runs = root["parallel_evolution_runs"] as? [[String: Any]] else { return [:] }
    var result: [String: [String: Any]] = [:]
    for run in runs.reversed() {
        guard let workers = run["workers"] as? [[String: Any]] else { continue }
        for worker in workers.reversed() {
            guard let field = text(worker["field"]), result[field] == nil else { continue }
            result[field] = worker
        }
    }
    return result
}

func latestVerifierByField(_ root: [String: Any]) -> [String: [String: Any]] {
    guard let results = root["field_verifier_results"] as? [[String: Any]] else { return [:] }
    var latest: [String: [String: Any]] = [:]
    for result in results.reversed() {
        guard let field = text(result["field"]), latest[field] == nil else { continue }
        latest[field] = result
    }
    return latest
}

func latestMiniTaskForField(_ root: [String: Any], field: String) -> String? {
    guard let traces = root["feed_traces"] as? [[String: Any]] else { return nil }
    for trace in traces.reversed() {
        let traceField = text(trace["field"]) ?? text(dict(trace["metadata"])?["field_pack"])
        guard traceField == field else { continue }
        if let task = text(dict(trace["metadata"])?["field_mini_task"]) {
            return task
        }
    }
    return nil
}

func workerState(from status: String?) -> String {
    switch status?.lowercased() {
    case "satisfied":
        return "feed"
    case "failed", "unsupported":
        return "blocked"
    case "partial":
        return "harness"
    default:
        return "heartbeat"
    }
}

func pendingQueuedNeedQueries(_ root: [String: Any]) -> [Int: String] {
    guard let items = root["need_queue"] as? [[String: Any]] else { return [:] }
    var queries: [Int: String] = [:]
    for item in items {
        guard (text(item["status"]) ?? "pending") == "pending",
              let index = int(item["index"]),
              let need = dict(item["need"]),
              let query = text(need["query"]) else { continue }
        queries[index] = query
    }
    return queries
}

func firstNonEmpty(_ values: [String]) -> String? {
    values.first { !$0.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty }
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

func stringArray(_ value: Any?) -> [String] {
    guard let array = value as? [Any] else { return [] }
    return array.compactMap(text)
}

func int(_ value: Any?) -> Int? {
    if let value = value as? Int { return value }
    if let value = value as? NSNumber { return value.intValue }
    if let value = value as? String { return Int(value) }
    return nil
}

func bool(_ value: Any?) -> Bool? {
    if let value = value as? Bool { return value }
    if let value = value as? NSNumber { return value.boolValue }
    if let value = value as? String {
        switch value.trimmingCharacters(in: .whitespacesAndNewlines).lowercased() {
        case "true", "yes", "1":
            return true
        case "false", "no", "0":
            return false
        default:
            return nil
        }
    }
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

        let palette = colors(for: displayState())
        let background = NSBezierPath(roundedRect: bounds.insetBy(dx: 8, dy: 8), xRadius: 18, yRadius: 18)
        NSColor(calibratedWhite: 1, alpha: 0.88).setFill()
        background.fill()
        NSColor(calibratedWhite: 0.82, alpha: 0.72).setStroke()
        background.lineWidth = 1
        background.stroke()

        drawGoalPill(palette: palette)
        let needText = displayNeed()
        if showNeedBubble(text: needText) {
            drawNeedBubble(text: needText, color: palette.head)
        }
        if showWorkBubbles() {
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

    private func drawNeedBubble(text: String, color: NSColor) {
        let text = clipped(text, max: 72)
        let rect = NSRect(x: 12, y: bounds.maxY - 126, width: 178, height: 68)
        let pulse = needBubblePulse()
        let halo = NSBezierPath(roundedRect: rect.insetBy(dx: -2 - pulse * 3, dy: -2 - pulse * 2), xRadius: 15, yRadius: 15)
        color.withAlphaComponent(0.10 + pulse * 0.08).setFill()
        halo.fill()
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

    private func needBubblePulse() -> CGFloat {
        0.5 + 0.5 * sin(tick * 1.8)
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
        let text = "\(palette.fallback) \(palette.label)\(suffix) · \(displaySource())"
        let attrs: [NSAttributedString.Key: Any] = [
            .font: NSFont.systemFont(ofSize: 12, weight: .semibold),
            .foregroundColor: NSColor(calibratedWhite: 0.16, alpha: 1),
        ]
        let clippedText = clipped(text, max: 40)
        let size = clippedText.size(withAttributes: attrs)
        clippedText.draw(at: NSPoint(x: bounds.midX - size.width / 2, y: 20), withAttributes: attrs)
    }

    private func displayNeed() -> String {
        if waitingObserverSlot() { return "" }
        if workerIndex < snapshot.workerNeeds.count {
            let value = snapshot.workerNeeds[workerIndex].trimmingCharacters(in: .whitespacesAndNewlines)
            if !value.isEmpty { return value }
        }
        return snapshot.need
    }

    private func showNeedBubble(text: String) -> Bool {
        if waitingObserverSlot() { return false }
        guard !text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return false }
        return snapshot.showNeedBubble
    }

    private func displaySource() -> String {
        if waitingObserverSlot() { return "observer-slot" }
        if workerIndex < snapshot.workerSources.count {
            let value = snapshot.workerSources[workerIndex].trimmingCharacters(in: .whitespacesAndNewlines)
            if !value.isEmpty { return value }
        }
        return snapshot.source
    }

    private func displayState() -> String {
        if waitingObserverSlot() { return "heartbeat" }
        let workerState = workerDisplayState()
        if snapshot.state == "action" {
            if workerIndex == 0 || workerFresh() { return "action" }
            return workerState ?? "heartbeat"
        }
        if let workerState { return workerState }
        return snapshot.state
    }

    private func workerDisplayState() -> String? {
        guard workerFresh() else { return nil }
        if workerIndex < snapshot.workerStates.count {
            let value = snapshot.workerStates[workerIndex].trimmingCharacters(in: .whitespacesAndNewlines)
            if !value.isEmpty { return value }
        }
        return nil
    }

    private func showWorkBubbles() -> Bool {
        if snapshot.showActionBubbles { return workerIndex == 0 || workerFresh() }
        return workerFresh() && ["harness", "evolution", "feed", "success", "blocked"].contains(displayState())
    }

    private func workerFresh() -> Bool {
        guard workerIndex < snapshot.workerUpdatedAt.count else { return false }
        return freshTimestamp(snapshot.workerUpdatedAt[workerIndex])
    }

    private func waitingObserverSlot() -> Bool {
        guard snapshot.workers > 1 else { return false }
        let workerDataCount = [
            snapshot.workerNeeds.count,
            snapshot.workerSources.count,
            snapshot.workerStates.count,
            snapshot.workerUpdatedAt.count,
        ].max() ?? 0
        return workerIndex >= workerDataCount
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
        let detail = snapshot.fieldPool.isEmpty ? snapshot.goal : "\(snapshot.goal)\n\n\(snapshot.fieldPool)"
        clipped(detail, max: 300).draw(in: NSRect(x: rect.minX + 16, y: rect.minY + 18, width: rect.width - 32, height: rect.height - 58), withAttributes: attrs)
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
            window.orderFrontRegardless()
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
app.run()
