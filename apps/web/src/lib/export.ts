import type { Entry, Session } from "./api";

export interface ExportOptions {
	includeSessionHeader?: boolean;
	includeTimestamps?: boolean;
	mapCodeBlocks?: boolean;
}

const VOICE_COMMANDS: { pattern: RegExp; replacement: string }[] = [
	{ pattern: /\bnewline\b/gi, replacement: "⏎" },
	{ pattern: /\bnew paragraph\b/gi, replacement: "⏎⏎" },
	{ pattern: /\btab\b/gi, replacement: "⇥" },
	{ pattern: /\bperiod\b/gi, replacement: "." },
	{ pattern: /\bcomma\b/gi, replacement: "," },
	{ pattern: /\bcolon\b/gi, replacement: ":" },
	{ pattern: /\bsemicolon\b/gi, replacement: ";" },
	{ pattern: /\bopen quote\b/gi, replacement: '"' },
	{ pattern: /\bclose quote\b/gi, replacement: '"' },
	{ pattern: /\bbacktick\b/gi, replacement: "`" },
	{ pattern: /\bcode block\b/gi, replacement: "```" },
];

function resetRegexState() {
	for (const cmd of VOICE_COMMANDS) {
		cmd.pattern.lastIndex = 0;
	}
}

export function hasVoiceCommands(text: string): boolean {
	resetRegexState();
	return VOICE_COMMANDS.some((cmd) => cmd.pattern.test(text));
}

export function processVoiceCommands(text: string): string {
	resetRegexState();
	let processed = text;
	const sortedCommands = [...VOICE_COMMANDS].sort(
		(a, b) => b.replacement.length - a.replacement.length
	);
	for (const cmd of sortedCommands) {
		processed = processed.replace(cmd.pattern, cmd.replacement);
	}
	return processed;
}

function formatTimestamp(timestamp: number): string {
	const date = new Date(timestamp);
	return date.toISOString();
}

function formatDuration(start: number, end: number): string {
	const totalSeconds = Math.floor((end - start) / 1000);
	const hours = Math.floor(totalSeconds / 3600);
	const minutes = Math.floor((totalSeconds % 3600) / 60);
	const seconds = totalSeconds % 60;

	return `${hours.toString().padStart(2, "0")}:${minutes
		.toString()
		.padStart(2, "0")}:${seconds.toString().padStart(2, "0")}`;
}

function formatSessionHeader(session: Session): string {
	const start = new Date(session.started_at).toISOString();
	const end = session.ended_at
		? new Date(session.ended_at).toISOString()
		: "ongoing";
	return `Session ${session.id} — ${start} → ${end}, model: ${session.model_profile}, translated: ${session.translated}`;
}

function formatMdTimestamp(start: number, end: number): string {
	return `> [${formatDuration(start, end)}]`;
}

export function exportToTxt(
	entries: Entry[],
	session: Session,
	options: ExportOptions = {}
): string {
	const { includeTimestamps = true, includeSessionHeader = true } = options;
	const parts: string[] = [];

	if (includeSessionHeader) {
		parts.push(formatSessionHeader(session));
	}

	const sortedEntries = [...entries].sort(
		(a, b) => a.started_at - b.started_at
	);

	for (const entry of sortedEntries) {
		if (includeTimestamps) {
			parts.push(`[${formatTimestamp(entry.started_at)}] ${entry.text}`);
		} else {
			parts.push(entry.text);
		}
	}

	return parts.join("\n");
}

export function exportToMd(
	entries: Entry[],
	session: Session,
	options: ExportOptions = {}
): string {
	const {
		includeTimestamps = true,
		includeSessionHeader = true,
		mapCodeBlocks = true,
	} = options;
	const parts: string[] = [];

	if (includeSessionHeader) {
		parts.push(`# Session ${session.id}`);
		parts.push("");
		parts.push(`- **Started**: ${new Date(session.started_at).toISOString()}`);
		if (session.ended_at) {
			parts.push(`- **Ended**: ${new Date(session.ended_at).toISOString()}`);
		}
		parts.push(`- **Mode**: ${session.mode}`);
		parts.push(`- **Model**: ${session.model_profile}`);
		parts.push(`- **Translated**: ${session.translated}`);
		parts.push("");
	}

	const sortedEntries = [...entries].sort(
		(a, b) => a.started_at - b.started_at
	);

	for (const entry of sortedEntries) {
		if (includeTimestamps) {
			parts.push(formatMdTimestamp(entry.started_at, entry.ended_at));
		}

		if (mapCodeBlocks && hasVoiceCommands(entry.text)) {
			const processed = processVoiceCommands(entry.text);
			parts.push("```");
			parts.push(processed);
			parts.push("```");
		} else {
			parts.push(entry.text);
		}

		parts.push("");
	}

	return parts.join("\n");
}

export function exportEntries(
	entries: Entry[],
	session: Session,
	format: "txt" | "md",
	options: ExportOptions = {}
): string {
	if (format === "txt") {
		return exportToTxt(entries, session, options);
	}
	return exportToMd(entries, session, options);
}

export function createExportBlob(content: string, format: "txt" | "md"): Blob {
	const mimeType = format === "txt" ? "text/plain" : "text/markdown";
	return new Blob([content], { type: mimeType });
}

export function downloadExport(
	content: string,
	format: "txt" | "md",
	filename?: string
): void {
	const blob = createExportBlob(content, format);
	const url = URL.createObjectURL(blob);
	const a = document.createElement("a");
	a.href = url;
	a.download = filename || `export-${Date.now()}.${format}`;
	document.body.appendChild(a);
	a.click();
	document.body.removeChild(a);
	URL.revokeObjectURL(url);
}
