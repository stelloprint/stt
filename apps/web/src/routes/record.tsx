import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";
import { toast } from "sonner";
import { api, type SessionCreate, type SessionMode } from "../lib/api";

const WORDS_REGEX = /\s+/;

export const Route = createFileRoute("/record")({
	component: RecordComponent,
});

function RecordComponent() {
	const [mode, setMode] = useState<SessionMode>("hold");
	const [recording, setRecording] = useState(false);
	const [sessionId, setSessionId] = useState<string | null>(null);
	const [transcript, setTranscript] = useState("");

	const startSession = async () => {
		const id = crypto.randomUUID();
		const session: SessionCreate = {
			id,
			mode,
			started_at: Date.now(),
			language: null,
			model_profile: "multilingual-small",
			translated: true,
			app_name: null,
		};

		try {
			await api.createSession(session);
			setSessionId(id);
			setRecording(true);
			setTranscript("");
			toast.success("Session started");
		} catch {
			toast.error("Failed to start session");
		}
	};

	const stopSession = async () => {
		if (!sessionId) {
			return;
		}

		try {
			const text = transcript.trim();
			const charsCount = text.length;
			const wordsCount = text.split(WORDS_REGEX).filter(Boolean).length;

			await api.updateSession(sessionId, Date.now(), charsCount, wordsCount);
			setRecording(false);
			setSessionId(null);
			toast.success("Session stopped");
		} catch {
			toast.error("Failed to stop session");
		}
	};

	const addEntry = async () => {
		if (!(sessionId && transcript.trim())) {
			return;
		}

		const entry = {
			id: crypto.randomUUID(),
			session_id: sessionId,
			started_at: Date.now(),
			ended_at: Date.now(),
			text: transcript,
			source: mode,
			typed: false,
		};

		try {
			await api.createEntry(entry);
			setTranscript("");
			toast.success("Entry added");
		} catch {
			toast.error("Failed to add entry");
		}
	};

	return (
		<div className="container mx-auto max-w-3xl px-4 py-2">
			<h1 className="mb-6 font-bold text-2xl">Record</h1>

			<div className="space-y-6">
				<section className="rounded-lg border p-4">
					<h2 className="mb-4 font-medium">Session Mode</h2>
					<div className="flex gap-4">
						<label className="flex items-center gap-2">
							<input
								checked={mode === "hold"}
								disabled={recording}
								name="mode"
								onChange={() => setMode("hold")}
								type="radio"
								value="hold"
							/>
							Hold
						</label>
						<label className="flex items-center gap-2">
							<input
								checked={mode === "toggle"}
								disabled={recording}
								name="mode"
								onChange={() => setMode("toggle")}
								type="radio"
								value="toggle"
							/>
							Toggle
						</label>
						<label className="flex items-center gap-2">
							<input
								checked={mode === "record"}
								disabled={recording}
								name="mode"
								onChange={() => setMode("record")}
								type="radio"
								value="record"
							/>
							Record
						</label>
					</div>
				</section>

				<section className="rounded-lg border p-4">
					<h2 className="mb-4 font-medium">Recording</h2>
					{recording ? (
						<div className="space-y-4">
							<div className="flex items-center gap-2">
								<span className="h-3 w-3 animate-pulse rounded-full bg-red-500" />
								<span>Recording...</span>
							</div>
							<textarea
								className="w-full rounded border p-2"
								onChange={(e) => setTranscript(e.target.value)}
								placeholder="Transcribed text will appear here..."
								rows={6}
								value={transcript}
							/>
							<div className="flex gap-2">
								<button
									className="rounded bg-blue-600 px-4 py-2 text-white hover:bg-blue-700 disabled:opacity-50"
									disabled={!transcript.trim()}
									onClick={addEntry}
									type="button"
								>
									Add Entry
								</button>
								<button
									className="rounded bg-red-600 px-4 py-2 text-white hover:bg-red-700"
									onClick={stopSession}
									type="button"
								>
									Stop Session
								</button>
							</div>
						</div>
					) : (
						<button
							className="rounded bg-green-600 px-4 py-2 text-white hover:bg-green-700"
							onClick={startSession}
							type="button"
						>
							Start Session
						</button>
					)}
				</section>
			</div>
		</div>
	);
}
