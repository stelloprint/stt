import { createFileRoute } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { api } from "../lib/api";

export const Route = createFileRoute("/")({
	component: HomeComponent,
});

const TITLE_TEXT = `
 ██████╗ ███████╗████████╗████████╗███████╗██████╗
 ██╔══██╗██╔════╝╚══██╔══╝╚══██╔══╝██╔════╝██╔══██╗
 ██████╔╝█████╗     ██║      ██║   █████╗  ██████╔╝
 ██╔══██╗██╔══╝     ██║      ██║   ██╔══╝  ██╔══██╗
 ██████╔╝███████╗   ██║      ██║   ███████╗██║  ██║
 ╚═════╝ ╚══════╝   ╚═╝      ╚═╝   ╚══════╝╚═╝  ╚═╝

 ████████╗    ███████╗████████╗ █████╗  ██████╗██╗  ██╗
 ╚══██╔══╝    ██╔════╝╚══██╔══╝██╔══██╗██╔════╝██║ ██╔╝
    ██║       ███████╗   ██║   ███████║██║     █████╔╝
    ██║       ╚════██║   ██║   ██╔══██║██║     ██╔═██╗
    ██║       ███████║   ██║   ██║  ██║╚██████╗██║  ██╗
    ╚═╝       ╚══════╝   ╚═╝   ╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝
 `;

function HomeComponent() {
	const [status, setStatus] = useState<"loading" | "connected" | "error">(
		"loading"
	);
	const [sessionCount, setSessionCount] = useState<number | null>(null);
	const [entryCount, setEntryCount] = useState<number | null>(null);

	useEffect(() => {
		api
			.getAllSessions()
			.then((sessions) => {
				setSessionCount(sessions.length);
				return api.getAllEntries();
			})
			.then((entries) => {
				setEntryCount(entries.length);
				setStatus("connected");
			})
			.catch(() => {
				setStatus("error");
			});
	}, []);

	return (
		<div className="container mx-auto max-w-3xl px-4 py-2">
			<pre className="overflow-x-auto font-mono text-sm">{TITLE_TEXT}</pre>
			<div className="grid gap-6">
				<section className="rounded-lg border p-4">
					<h2 className="mb-2 font-medium">API Status</h2>
					<div className="flex items-center gap-2">
						{status === "loading" && (
							<span className="text-muted-foreground">Loading...</span>
						)}
						{status === "connected" && (
							<>
								<span className="h-2 w-2 rounded-full bg-green-500" />
								<span className="text-green-600">Connected</span>
							</>
						)}
						{status === "error" && (
							<>
								<span className="h-2 w-2 rounded-full bg-red-500" />
								<span className="text-red-600">Error</span>
							</>
						)}
					</div>
					{status === "connected" && (
						<p className="mt-2 text-muted-foreground text-sm">
							{sessionCount} sessions, {entryCount} entries
						</p>
					)}
				</section>
			</div>
		</div>
	);
}
