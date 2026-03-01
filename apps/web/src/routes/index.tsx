import { createFileRoute } from "@tanstack/react-router";
import { Activity, Database, Mic, MicOff, Settings } from "lucide-react";
import { useEffect, useState } from "react";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { api } from "@/lib/api";

export const Route = createFileRoute("/")({
	loader: async () => {
		const prefs = await api.preferences.get();
		const sessions = await api.sessions.getAll();
		const latestSession = sessions[0] ?? null;
		return { prefs, latestSession };
	},
	component: HomeComponent,
});

function HomeComponent() {
	const { prefs, latestSession } = Route.useLoaderData();
	const [isRecording] = useState(false);
	const [micLevel, setMicLevel] = useState(0);

	useEffect(() => {
		const interval = setInterval(() => {
			setMicLevel(Math.random() * 100);
		}, 500);
		return () => clearInterval(interval);
	}, []);

	const getModeLabel = (mode: string) => {
		switch (mode) {
			case "hold":
				return "Hold (push-to-talk)";
			case "toggle":
				return "Toggle";
			case "record":
				return "Record";
			default:
				return mode;
		}
	};

	const getModelLabel = (profile: string) => {
		switch (profile) {
			case "small.en":
				return "English (Small)";
			case "multilingual-small":
				return "Multilingual (Small)";
			case "multilingual-medium":
				return "Multilingual (Medium)";
			default:
				return profile;
		}
	};

	return (
		<div className="container mx-auto max-w-3xl px-4 py-2">
			<div className="mb-6 flex items-center justify-between">
				<h1 className="font-medium text-lg">HUD</h1>
				<div className="flex items-center gap-2">
					{isRecording ? (
						<span className="flex items-center gap-1 text-red-500 text-xs">
							<span className="h-2 w-2 animate-pulse rounded-full bg-red-500" />
							Recording
						</span>
					) : (
						<span className="flex items-center gap-1 text-muted-foreground text-xs">
							<span className="h-2 w-2 rounded-full bg-muted-foreground/30" />
							Idle
						</span>
					)}
				</div>
			</div>

			<div className="grid gap-4">
				<Card>
					<CardHeader className="pb-2">
						<CardTitle className="flex items-center gap-2 text-sm">
							<Activity className="h-4 w-4" />
							Status
						</CardTitle>
					</CardHeader>
					<CardContent>
						<div className="grid grid-cols-2 gap-4">
							<div>
								<div className="text-muted-foreground text-xs">Mode</div>
								<div className="font-medium">{getModeLabel(prefs.mode)}</div>
							</div>
							<div>
								<div className="text-muted-foreground text-xs">Model</div>
								<div className="font-medium">
									{getModelLabel(prefs.model_profile)}
								</div>
							</div>
							<div>
								<div className="text-muted-foreground text-xs">Mic Level</div>
								<div className="flex items-center gap-2">
									{micLevel > 10 ? (
										<Mic className="h-3 w-3 text-green-500" />
									) : (
										<MicOff className="h-3 w-3 text-muted-foreground" />
									)}
									<div className="h-2 flex-1 overflow-hidden rounded bg-muted">
										<div
											className="h-full bg-green-500 transition-all"
											style={{ width: `${micLevel}%` }}
										/>
									</div>
									<span className="text-xs">{Math.round(micLevel)}%</span>
								</div>
							</div>
							<div>
								<div className="text-muted-foreground text-xs">
									Voice Commands
								</div>
								<div className="font-medium">
									{prefs.voice_commands.enabled ? "Enabled" : "Disabled"}
								</div>
							</div>
						</div>
					</CardContent>
				</Card>

				<Card>
					<CardHeader className="pb-2">
						<CardTitle className="flex items-center gap-2 text-sm">
							<Database className="h-4 w-4" />
							Session Stats
						</CardTitle>
					</CardHeader>
					<CardContent>
						{latestSession ? (
							<div className="grid grid-cols-2 gap-4">
								<div>
									<div className="text-muted-foreground text-xs">
										Current Session
									</div>
									<div className="font-mono text-xs">
										{latestSession.id.slice(0, 8)}
									</div>
								</div>
								<div>
									<div className="text-muted-foreground text-xs">Words</div>
									<div className="font-medium">{latestSession.words_count}</div>
								</div>
								<div>
									<div className="text-muted-foreground text-xs">
										Characters
									</div>
									<div className="font-medium">{latestSession.chars_count}</div>
								</div>
								<div>
									<div className="text-muted-foreground text-xs">Duration</div>
									<div className="font-medium">
										{latestSession.ended_at
											? `${Math.round(
													(latestSession.ended_at - latestSession.started_at) /
														1000
												)}s`
											: "Active"}
									</div>
								</div>
							</div>
						) : (
							<p className="text-muted-foreground text-sm">No active session</p>
						)}
					</CardContent>
				</Card>

				<Card>
					<CardHeader className="pb-2">
						<CardTitle className="flex items-center gap-2 text-sm">
							<Settings className="h-4 w-4" />
							Quick Settings
						</CardTitle>
					</CardHeader>
					<CardContent>
						<div className="grid grid-cols-2 gap-4">
							<div>
								<div className="text-muted-foreground text-xs">Hotkeys</div>
								<div className="text-sm">
									{prefs.hotkeys.left_chord && "L "}
									{prefs.hotkeys.right_chord && "R"} Cmd+Opt
								</div>
							</div>
							<div>
								<div className="text-muted-foreground text-xs">
									Silence Timeout
								</div>
								<div className="text-sm">{prefs.silence_seconds}s</div>
							</div>
							<div>
								<div className="text-muted-foreground text-xs">Translate</div>
								<div className="text-sm">
									{prefs.translate_to_english ? "To English" : "Original"}
								</div>
							</div>
							<div>
								<div className="text-muted-foreground text-xs">Typing</div>
								<div className="text-sm">
									{prefs.typing.newline_at_end ? "+newline" : "no newline"}
									{prefs.typing.throttle_ms > 0 &&
										` (${prefs.typing.throttle_ms}ms)`}
								</div>
							</div>
						</div>
					</CardContent>
				</Card>
			</div>
		</div>
	);
}
