import { createFileRoute } from "@tanstack/react-router";

import {
	Clock,
	Download,
	FileAudio,
	Play,
	RotateCcw,
	Square,
} from "lucide-react";
import { useEffect, useState } from "react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { api, type Entry, type Session } from "@/lib/api";

export const Route = createFileRoute("/record")({
	loader: async () => {
		const sessions = await api.sessions.getAll();
		const activeSessions = sessions.filter((s: Session) => !s.ended_at);
		const entries = await api.entries.getAll();
		return { sessions, activeSessions, entries };
	},
	component: RecordComponent,
});

function RecordComponent() {
	const { sessions, activeSessions } = Route.useLoaderData();
	const [isRecording, setIsRecording] = useState(activeSessions.length > 0);
	const [currentSession, setCurrentSession] = useState<Session | null>(
		activeSessions[0] ?? null
	);
	const [sessionEntries, setSessionEntries] = useState<Entry[]>([]);
	const [fileRotationCount] = useState(0);

	useEffect(() => {
		if (currentSession) {
			api.entries.getBySession(currentSession.id).then(setSessionEntries);
		}
	}, [currentSession]);

	const handleStartRecording = async () => {
		try {
			const session = await api.sessions.create({
				id: crypto.randomUUID(),
				mode: "record",
				started_at: Date.now(),
				language: null,
				model_profile: "multilingual-small",
				translated: true,
				app_name: null,
			});
			setCurrentSession(session);
			setIsRecording(true);
		} catch (error) {
			console.error("Failed to start recording:", error);
		}
	};

	const handleStopRecording = async () => {
		if (!currentSession) {
			return;
		}

		try {
			await api.sessions.update(
				currentSession.id,
				Date.now(),
				undefined,
				undefined
			);
			setIsRecording(false);
			setCurrentSession(null);
		} catch (error) {
			console.error("Failed to stop recording:", error);
		}
	};

	const handleExportTranscript = () => {
		const content = sessionEntries
			.map((e) => `[${new Date(e.started_at).toLocaleTimeString()}] ${e.text}`)
			.join("\n\n");

		const blob = new Blob([content], { type: "text/plain" });
		const url = URL.createObjectURL(blob);
		const a = document.createElement("a");
		a.href = url;
		a.download = `transcript-${new Date().toISOString().split("T")[0]}.txt`;
		document.body.appendChild(a);
		a.click();
		document.body.removeChild(a);
		URL.revokeObjectURL(url);
	};

	const formatDuration = (start: number, end?: number | null) => {
		const duration = end ? end - start : Date.now() - start;
		const seconds = Math.floor(duration / 1000);
		const minutes = Math.floor(seconds / 60);
		const hours = Math.floor(minutes / 60);

		if (hours > 0) {
			return `${hours}h ${minutes % 60}m ${seconds % 60}s`;
		}
		if (minutes > 0) {
			return `${minutes}m ${seconds % 60}s`;
		}
		return `${seconds}s`;
	};

	const getTotalStats = () => {
		const sessionData =
			isRecording && currentSession ? [...sessions, currentSession] : sessions;
		const totalWords = sessionData.reduce(
			(acc: number, s: Session) => acc + s.words_count,
			0
		);
		const totalChars = sessionData.reduce(
			(acc: number, s: Session) => acc + s.chars_count,
			0
		);
		return { totalWords, totalChars, totalSessions: sessionData.length };
	};

	const stats = getTotalStats();

	return (
		<div className="container mx-auto max-w-4xl px-4 py-2">
			<div className="mb-6 flex items-center justify-between">
				<h1 className="font-medium text-lg">Record</h1>
				<div className="flex items-center gap-2">
					{isRecording ? (
						<span className="flex items-center gap-1 text-red-500 text-xs">
							<span className="h-2 w-2 animate-pulse rounded-full bg-red-500" />
							Recording
						</span>
					) : (
						<span className="text-muted-foreground text-xs">Ready</span>
					)}
				</div>
			</div>

			<div className="grid gap-6">
				<Card>
					<CardHeader className="pb-2">
						<CardTitle className="flex items-center gap-2 text-sm">
							<FileAudio className="h-4 w-4" />
							Recording Controls
						</CardTitle>
					</CardHeader>
					<CardContent>
						<div className="flex items-center gap-4">
							{isRecording ? (
								<Button onClick={handleStopRecording} variant="destructive">
									<Square className="mr-2 h-4 w-4" />
									Stop Recording
								</Button>
							) : (
								<Button onClick={handleStartRecording}>
									<Play className="mr-2 h-4 w-4" />
									Start Recording
								</Button>
							)}
							{isRecording && currentSession && (
								<div className="flex items-center gap-4 text-sm">
									<span className="flex items-center gap-1">
										<Clock className="h-3 w-3" />
										{formatDuration(currentSession.started_at)}
									</span>
									<span className="flex items-center gap-1">
										<RotateCcw className="h-3 w-3" />
										{fileRotationCount} rotations
									</span>
								</div>
							)}
						</div>
					</CardContent>
				</Card>

				{isRecording && currentSession && (
					<Card>
						<CardHeader className="pb-2">
							<CardTitle className="text-sm">Live Transcript</CardTitle>
						</CardHeader>
						<CardContent>
							{sessionEntries.length === 0 ? (
								<p className="text-muted-foreground text-sm">
									Waiting for speech...
								</p>
							) : (
								<div className="grid gap-3">
									{sessionEntries.map((entry) => (
										<div className="text-sm" key={entry.id}>
											<span className="text-muted-foreground">
												[
												{formatDuration(
													currentSession.started_at,
													entry.started_at
												)}
												]{" "}
											</span>
											{entry.text}
										</div>
									))}
								</div>
							)}
						</CardContent>
					</Card>
				)}

				<Card>
					<CardHeader className="pb-2">
						<CardTitle className="flex items-center justify-between text-sm">
							<span>Session History</span>
							<Button
								disabled={sessions.length === 0}
								onClick={handleExportTranscript}
								size="sm"
								variant="secondary"
							>
								<Download className="mr-2 h-3 w-3" />
								Export All
							</Button>
						</CardTitle>
					</CardHeader>
					<CardContent>
						{sessions.length === 0 ? (
							<p className="text-muted-foreground text-sm">
								No recordings yet.
							</p>
						) : (
							<div className="grid gap-2">
								<div className="flex items-center justify-between rounded border p-2 text-xs">
									<div className="flex gap-4">
										<span className="font-medium">Total</span>
										<span>{stats.totalSessions} sessions</span>
										<span>{stats.totalWords} words</span>
										<span>{stats.totalChars} chars</span>
									</div>
								</div>
								{sessions.slice(0, 10).map((session: Session) => (
									<div
										className="flex items-center justify-between rounded border p-2 text-xs"
										key={session.id}
									>
										<div className="flex gap-4">
											<span className="font-mono">
												{session.id.slice(0, 8)}
											</span>
											<span className="text-muted-foreground">
												{new Date(session.started_at).toLocaleString()}
											</span>
										</div>
										<div className="flex gap-4">
											<span>{session.words_count} words</span>
											<span>
												{formatDuration(session.started_at, session.ended_at)}
											</span>
										</div>
									</div>
								))}
							</div>
						)}
					</CardContent>
				</Card>
			</div>
		</div>
	);
}
