import { createFileRoute } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import { api, type Entry, type Session } from "../lib/api";

export const Route = createFileRoute("/logs")({
	component: LogsComponent,
});

function LogsComponent() {
	const [sessions, setSessions] = useState<Session[]>([]);
	const [entries, setEntries] = useState<Entry[]>([]);
	const [searchQuery, setSearchQuery] = useState("");
	const [filteredEntries, setFilteredEntries] = useState<Entry[]>([]);
	const [loading, setLoading] = useState(true);

	useEffect(() => {
		Promise.all([api.getAllSessions(), api.getAllEntries()])
			.then(([sessions, entries]) => {
				setSessions(sessions);
				setEntries(entries);
				setFilteredEntries(entries);
			})
			.catch(() => {
				toast.error("Failed to load logs");
			})
			.finally(() => setLoading(false));
	}, []);

	useEffect(() => {
		if (searchQuery.trim()) {
			api
				.searchEntries(searchQuery)
				.then(setFilteredEntries)
				.catch(() => toast.error("Search failed"));
		} else {
			setFilteredEntries(entries);
		}
	}, [searchQuery, entries]);

	const handleDeleteSession = async (id: string) => {
		try {
			await api.deleteSession(id);
			setSessions((prev) => prev.filter((s) => s.id !== id));
			setEntries((prev) => prev.filter((e) => e.session_id !== id));
			setFilteredEntries((prev) => prev.filter((e) => e.session_id !== id));
			toast.success("Session deleted");
		} catch {
			toast.error("Failed to delete session");
		}
	};

	const _getSessionMode = (sessionId: string) => {
		const session = sessions.find((s) => s.id === sessionId);
		return session?.mode || "unknown";
	};

	const formatDate = (timestamp: number) => {
		return new Date(timestamp).toLocaleString();
	};

	if (loading) {
		return (
			<div className="container mx-auto max-w-3xl px-4 py-2">
				<p>Loading...</p>
			</div>
		);
	}

	return (
		<div className="container mx-auto max-w-3xl px-4 py-2">
			<h1 className="mb-6 font-bold text-2xl">Logs</h1>

			<div className="mb-4">
				<input
					className="w-full rounded border p-2"
					onChange={(e) => setSearchQuery(e.target.value)}
					placeholder="Search entries..."
					type="text"
					value={searchQuery}
				/>
			</div>

			<div className="space-y-4">
				{filteredEntries.length === 0 ? (
					<p className="text-muted-foreground">No entries found</p>
				) : (
					filteredEntries.map((entry) => (
						<div className="rounded-lg border p-4" key={entry.id}>
							<div className="mb-2 flex items-center justify-between">
								<span className="text-muted-foreground text-sm">
									{formatDate(entry.started_at)} - {entry.source}
								</span>
								<span
									className={`rounded px-2 py-0.5 text-xs ${
										entry.typed
											? "bg-green-100 text-green-800"
											: "bg-yellow-100 text-yellow-800"
									}`}
								>
									{entry.typed ? "typed" : "pending"}
								</span>
							</div>
							<p className="whitespace-pre-wrap">{entry.text}</p>
						</div>
					))
				)}
			</div>

			{sessions.length > 0 && (
				<div className="mt-8">
					<h2 className="mb-4 font-semibold text-xl">Sessions</h2>
					<div className="space-y-2">
						{sessions.map((session) => (
							<div
								className="flex items-center justify-between rounded border p-3"
								key={session.id}
							>
								<div>
									<p className="font-mono text-sm">{session.id}</p>
									<p className="text-muted-foreground text-xs">
										{session.mode} - {formatDate(session.started_at)}
									</p>
									<p className="text-muted-foreground text-xs">
										{session.chars_count} chars, {session.words_count} words
									</p>
								</div>
								<button
									className="text-red-500 hover:text-red-700"
									onClick={() => handleDeleteSession(session.id)}
									type="button"
								>
									Delete
								</button>
							</div>
						))}
					</div>
				</div>
			)}
		</div>
	);
}
