import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { api, type Entry, type Session } from "@/lib/api";

export const Route = createFileRoute("/logs")({
	loader: async () => {
		const [sessions, entries] = await Promise.all([
			api.sessions.getAll(),
			api.entries.getAll(),
		]);
		return { sessions, entries };
	},
	component: LogsComponent,
});

function LogsComponent() {
	const { sessions, entries }: { entries: Entry[]; sessions: Session[] } =
		Route.useLoaderData();
	const [searchQuery, setSearchQuery] = useState("");
	const [filteredEntries, setFilteredEntries] = useState<Entry[]>(entries);
	const [isSearching, setIsSearching] = useState(false);

	const handleSearch = async () => {
		if (!searchQuery.trim()) {
			setFilteredEntries(entries);
			return;
		}

		setIsSearching(true);
		try {
			const results = await api.entries.search(searchQuery);
			setFilteredEntries(results);
		} catch (error) {
			toast.error("Search failed", {
				description: error instanceof Error ? error.message : "Unknown error",
			});
		} finally {
			setIsSearching(false);
		}
	};

	const handleExport = () => {
		const content = filteredEntries
			.map((e) => `[${new Date(e.started_at).toISOString()}] ${e.text}`)
			.join("\n");

		const blob = new Blob([content], { type: "text/plain" });
		const url = URL.createObjectURL(blob);
		const a = document.createElement("a");
		a.href = url;
		a.download = `stt-logs-${new Date().toISOString().split("T")[0]}.txt`;
		document.body.appendChild(a);
		a.click();
		document.body.removeChild(a);
		URL.revokeObjectURL(url);
		toast.success("Exported logs");
	};

	const formatDate = (timestamp: number) => {
		return new Date(timestamp).toLocaleString();
	};

	const formatDuration = (start: number, end: number) => {
		const seconds = Math.round((end - start) / 1000);
		if (seconds < 60) {
			return `${seconds}s`;
		}
		const minutes = Math.floor(seconds / 60);
		const remainingSeconds = seconds % 60;
		return `${minutes}m ${remainingSeconds}s`;
	};

	const getModeLabel = (mode: string) => {
		switch (mode) {
			case "hold":
				return "Hold";
			case "toggle":
				return "Toggle";
			case "record":
				return "Record";
			default:
				return mode;
		}
	};

	const sessionMap = new Map<string, Session>(
		sessions.map((s: Session) => [s.id, s])
	);

	return (
		<div className="container mx-auto max-w-5xl px-4 py-2">
			<div className="mb-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
				<h1 className="font-medium text-lg">Logs</h1>
				<div className="flex gap-2">
					<Input
						className="w-64"
						onChange={(e) => setSearchQuery(e.target.value)}
						onKeyDown={(e) => {
							if (e.key === "Enter") {
								handleSearch();
							}
						}}
						placeholder="Search entries..."
						value={searchQuery}
					/>
					<Button
						disabled={isSearching}
						onClick={handleSearch}
						variant="secondary"
					>
						Search
					</Button>
					<Button onClick={handleExport} variant="secondary">
						Export
					</Button>
				</div>
			</div>

			<div className="grid gap-4">
				<Card>
					<CardHeader>
						<CardTitle>Sessions ({sessions.length})</CardTitle>
					</CardHeader>
					<CardContent>
						{sessions.length === 0 ? (
							<p className="text-muted-foreground text-sm">No sessions yet.</p>
						) : (
							<div className="grid gap-2">
								{sessions.map((session: Session) => (
									<div
										className="flex items-center justify-between rounded border p-2 text-xs"
										key={session.id}
									>
										<div className="flex gap-4">
											<span className="font-mono">
												{session.id.slice(0, 8)}
											</span>
											<span>{getModeLabel(session.mode)}</span>
											<span className="text-muted-foreground">
												{formatDate(session.started_at)}
											</span>
										</div>
										<div className="flex gap-4">
											<span>{session.words_count} words</span>
											<span>{session.chars_count} chars</span>
										</div>
									</div>
								))}
							</div>
						)}
					</CardContent>
				</Card>

				<Card>
					<CardHeader>
						<CardTitle>Entries ({filteredEntries.length})</CardTitle>
					</CardHeader>
					<CardContent>
						{filteredEntries.length === 0 ? (
							<p className="text-muted-foreground text-sm">
								{searchQuery ? "No matching entries." : "No entries yet."}
							</p>
						) : (
							<div className="grid gap-2">
								{filteredEntries.map((entry: Entry) => {
									const session = sessionMap.get(entry.session_id);
									return (
										<div className="rounded border p-3 text-xs" key={entry.id}>
											<div className="mb-1 flex items-center justify-between text-muted-foreground">
												<span>
													{formatDate(entry.started_at)} (
													{formatDuration(entry.started_at, entry.ended_at)})
												</span>
												<span>{getModeLabel(entry.source)}</span>
											</div>
											<p className="wrap-break-word">{entry.text}</p>
											{session && (
												<div className="mt-1 text-muted-foreground">
													Session: {session.id.slice(0, 8)} | Typed:{" "}
													{entry.typed ? "Yes" : "No"}
												</div>
											)}
										</div>
									);
								})}
							</div>
						)}
					</CardContent>
				</Card>
			</div>
		</div>
	);
}
