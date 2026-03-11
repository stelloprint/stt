import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";

import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import {
	Card,
	CardAction,
	CardContent,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
	type ActivationMode,
	api,
	type ModelProfile,
	type ModelStatus,
	type Preferences,
	type SilenceRms,
} from "@/lib/api";

export const Route = createFileRoute("/settings")({
	loader: async () => {
		const [prefs, modelStatuses] = await Promise.all([
			api.preferences.get(),
			api.models.getStatuses(),
		]);
		return { prefs, modelStatuses };
	},
	component: SettingsComponent,
});

const modelProfileLabels: Record<ModelProfile, string> = {
	"small.en": "English (Small)",
	"multilingual-small": "Multilingual (Small)",
	"multilingual-medium": "Multilingual (Medium)",
};

const silenceRmsLabels: Record<SilenceRms, string> = {
	low: "Low",
	medium: "Medium",
	high: "High",
};

function getModelStatusBadge(status: ModelStatus) {
	if (status.is_verified) {
		return (
			<span className="rounded-full bg-green-100 px-2 py-0.5 text-green-700 text-xs dark:bg-green-900 dark:text-green-300">
				Verified
			</span>
		);
	}
	if (status.file_exists) {
		return (
			<span className="rounded-full bg-yellow-100 px-2 py-0.5 text-xs text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300">
				Mismatch
			</span>
		);
	}
	return (
		<span className="rounded-full bg-red-100 px-2 py-0.5 text-red-700 text-xs dark:bg-red-900 dark:text-red-300">
			Missing
		</span>
	);
}

function SettingsComponent() {
	const { prefs, modelStatuses: initialModelStatuses } = Route.useLoaderData();
	const [modelStatuses, setModelStatuses] = useState(initialModelStatuses);

	const handleSave = async (formData: FormData) => {
		const newPrefs: Preferences = {
			hotkeys: {
				left_chord: formData.get("left_chord") === "on",
				right_chord: formData.get("right_chord") === "on",
			},
			mode: formData.get("mode") as ActivationMode,
			silence_seconds:
				Number.parseFloat(formData.get("silence_seconds") as string) || 3.0,
			silence_rms: formData.get("silence_rms") as SilenceRms,
			model_profile: formData.get("model_profile") as ModelProfile,
			translate_to_english: formData.get("translate_to_english") === "on",
			typing: {
				newline_at_end: formData.get("newline_at_end") === "on",
				throttle_ms:
					Number.parseInt(formData.get("throttle_ms") as string, 10) || 0,
			},
			voice_commands: {
				enabled: formData.get("voice_commands_enabled") === "on",
				map: {
					newline: formData.get("map_newline") as string,
					new_paragraph: formData.get("map_new_paragraph") as string,
					tab: formData.get("map_tab") as string,
					period: formData.get("map_period") as string,
					comma: formData.get("map_comma") as string,
					colon: formData.get("map_colon") as string,
					semicolon: formData.get("map_semicolon") as string,
					open_quote: formData.get("map_open_quote") as string,
					close_quote: formData.get("map_close_quote") as string,
					backtick: formData.get("map_backtick") as string,
					code_block: formData.get("map_code_block") as string,
				},
			},
			record: {
				chunk_seconds:
					Number.parseInt(formData.get("chunk_seconds") as string, 10) || 60,
				max_hours:
					Number.parseInt(formData.get("max_hours") as string, 10) || 8,
				max_file_gb:
					Number.parseInt(formData.get("max_file_gb") as string, 10) || 4,
			},
		};

		try {
			await api.preferences.update(newPrefs);
			toast.success("Settings saved");
		} catch (error) {
			toast.error("Failed to save settings", {
				description: error instanceof Error ? error.message : "Unknown error",
			});
		}
	};

	return (
		<div className="container mx-auto max-w-3xl px-4 py-2">
			<form action={handleSave}>
				<div className="grid gap-6">
					<Card>
						<CardHeader>
							<CardTitle>Activation</CardTitle>
						</CardHeader>
						<CardContent className="grid gap-4">
							<div className="grid grid-cols-2 gap-4">
								<div className="grid gap-2">
									<Label htmlFor="mode">Mode</Label>
									<select
										className="h-8 w-full rounded-none border border-input bg-transparent px-2.5 py-1 text-xs outline-none focus-visible:border-ring focus-visible:ring-1 focus-visible:ring-ring/50"
										defaultValue={prefs.mode}
										id="mode"
										name="mode"
									>
										<option value="hold">Hold</option>
										<option value="toggle">Toggle</option>
									</select>
								</div>
								<div className="grid gap-2">
									<Label htmlFor="silence_seconds">
										Silence timeout (seconds)
									</Label>
									<Input
										defaultValue={prefs.silence_seconds}
										id="silence_seconds"
										max="30"
										min="0.5"
										name="silence_seconds"
										step="0.5"
										type="number"
									/>
								</div>
							</div>
							<div className="grid gap-2">
								<Label>Hotkeys</Label>
								<div className="flex gap-4">
									<label
										className="flex items-center gap-2"
										htmlFor="left_chord"
									>
										<Checkbox
											defaultChecked={prefs.hotkeys.left_chord}
											id="left_chord"
											name="left_chord"
										/>
										<span className="text-xs">Left Cmd+Opt</span>
									</label>
									<label
										className="flex items-center gap-2"
										htmlFor="right_chord"
									>
										<Checkbox
											defaultChecked={prefs.hotkeys.right_chord}
											id="right_chord"
											name="right_chord"
										/>
										<span className="text-xs">Right Cmd+Opt</span>
									</label>
								</div>
							</div>
						</CardContent>
					</Card>

					<Card>
						<CardHeader>
							<CardTitle>Model</CardTitle>
						</CardHeader>
						<CardContent className="grid gap-4">
							<div className="grid grid-cols-2 gap-4">
								<div className="grid gap-2">
									<Label htmlFor="model_profile">Profile</Label>
									<select
										className="h-8 w-full rounded-none border border-input bg-transparent px-2.5 py-1 text-xs outline-none focus-visible:border-ring focus-visible:ring-1 focus-visible:ring-ring/50"
										defaultValue={prefs.model_profile}
										id="model_profile"
										name="model_profile"
									>
										{Object.entries(modelProfileLabels).map(
											([value, label]) => (
												<option key={value} value={value}>
													{label}
												</option>
											)
										)}
									</select>
								</div>
								<div className="grid gap-2">
									<Label htmlFor="silence_rms">Silence sensitivity</Label>
									<select
										className="h-8 w-full rounded-none border border-input bg-transparent px-2.5 py-1 text-xs outline-none focus-visible:border-ring focus-visible:ring-1 focus-visible:ring-ring/50"
										defaultValue={prefs.silence_rms}
										id="silence_rms"
										name="silence_rms"
									>
										{Object.entries(silenceRmsLabels).map(([value, label]) => (
											<option key={value} value={value}>
												{label}
											</option>
										))}
									</select>
								</div>
							</div>
							<label
								className="flex items-center gap-2"
								htmlFor="translate_to_english"
							>
								<Checkbox
									defaultChecked={prefs.translate_to_english}
									id="translate_to_english"
									name="translate_to_english"
								/>
								<span className="text-xs">Translate to English</span>
							</label>
						</CardContent>
					</Card>

					<Card>
						<CardHeader>
							<CardTitle>Model Status</CardTitle>
							<CardAction>
								<a
									className="text-muted-foreground text-xs underline hover:text-foreground"
									href="https://huggingface.co/ggerganov/whisper.cpp"
									rel="noopener noreferrer"
									target="_blank"
								>
									Download models
								</a>
							</CardAction>
						</CardHeader>
						<CardContent className="grid gap-3">
							{modelStatuses.map((status) => (
								<div
									className="grid grid-cols-[1fr_auto] items-center gap-3 rounded-md border p-3"
									key={status.profile}
								>
									<div className="grid gap-1">
										<div className="flex items-center gap-2">
											<span className="font-medium text-sm">
												{modelProfileLabels[status.profile as ModelProfile] ??
													status.profile}
											</span>
											{getModelStatusBadge(status)}
										</div>
										<span className="text-muted-foreground text-xs">
											{status.filename}
										</span>
									</div>
									<Button
										onClick={async () => {
											try {
												const updated = await api.models.verify(status.profile);
												setModelStatuses((prev) =>
													prev.map((m) =>
														m.profile === status.profile ? updated : m
													)
												);
												if (updated.is_verified) {
													toast.success(
														`${status.profile} verified successfully`
													);
												} else if (updated.file_exists) {
													toast.error(`${status.profile} SHA-256 mismatch`);
												} else {
													toast.error(`${status.profile} not found`);
												}
											} catch (error) {
												toast.error("Verification failed", {
													description:
														error instanceof Error
															? error.message
															: "Unknown error",
												});
											}
										}}
										size="sm"
										variant="outline"
									>
										Verify
									</Button>
								</div>
							))}
						</CardContent>
					</Card>

					<Card>
						<CardHeader>
							<CardTitle>Typing</CardTitle>
						</CardHeader>
						<CardContent className="grid gap-4">
							<div className="grid grid-cols-2 gap-4">
								<div className="grid gap-2">
									<Label htmlFor="throttle_ms">Throttle (ms)</Label>
									<Input
										defaultValue={prefs.typing.throttle_ms}
										id="throttle_ms"
										max="1000"
										min="0"
										name="throttle_ms"
										type="number"
									/>
								</div>
							</div>
							<label
								className="flex items-center gap-2"
								htmlFor="newline_at_end"
							>
								<Checkbox
									defaultChecked={prefs.typing.newline_at_end}
									id="newline_at_end"
									name="newline_at_end"
								/>
								<span className="text-xs">Append newline at end</span>
							</label>
						</CardContent>
					</Card>

					<Card>
						<CardHeader>
							<CardTitle>Voice Commands</CardTitle>
							<CardAction>
								<label
									className="flex items-center gap-2"
									htmlFor="voice_commands_enabled"
								>
									<Checkbox
										defaultChecked={prefs.voice_commands.enabled}
										id="voice_commands_enabled"
										name="voice_commands_enabled"
									/>
									<span className="text-xs">Enabled</span>
								</label>
							</CardAction>
						</CardHeader>
						<CardContent>
							<div className="grid grid-cols-2 gap-x-4 gap-y-2">
								<div className="grid gap-1">
									<Label className="text-xs" htmlFor="map_newline">
										New line
									</Label>
									<Input
										defaultValue={prefs.voice_commands.map.newline}
										id="map_newline"
										name="map_newline"
									/>
								</div>
								<div className="grid gap-1">
									<Label className="text-xs" htmlFor="map_new_paragraph">
										New paragraph
									</Label>
									<Input
										defaultValue={prefs.voice_commands.map.new_paragraph}
										id="map_new_paragraph"
										name="map_new_paragraph"
									/>
								</div>
								<div className="grid gap-1">
									<Label className="text-xs" htmlFor="map_tab">
										Tab
									</Label>
									<Input
										defaultValue={prefs.voice_commands.map.tab}
										id="map_tab"
										name="map_tab"
									/>
								</div>
								<div className="grid gap-1">
									<Label className="text-xs" htmlFor="map_period">
										Period
									</Label>
									<Input
										defaultValue={prefs.voice_commands.map.period}
										id="map_period"
										name="map_period"
									/>
								</div>
								<div className="grid gap-1">
									<Label className="text-xs" htmlFor="map_comma">
										Comma
									</Label>
									<Input
										defaultValue={prefs.voice_commands.map.comma}
										id="map_comma"
										name="map_comma"
									/>
								</div>
								<div className="grid gap-1">
									<Label className="text-xs" htmlFor="map_colon">
										Colon
									</Label>
									<Input
										defaultValue={prefs.voice_commands.map.colon}
										id="map_colon"
										name="map_colon"
									/>
								</div>
								<div className="grid gap-1">
									<Label className="text-xs" htmlFor="map_semicolon">
										Semicolon
									</Label>
									<Input
										defaultValue={prefs.voice_commands.map.semicolon}
										id="map_semicolon"
										name="map_semicolon"
									/>
								</div>
								<div className="grid gap-1">
									<Label className="text-xs" htmlFor="map_open_quote">
										Open quote
									</Label>
									<Input
										defaultValue={prefs.voice_commands.map.open_quote}
										id="map_open_quote"
										name="map_open_quote"
									/>
								</div>
								<div className="grid gap-1">
									<Label className="text-xs" htmlFor="map_close_quote">
										Close quote
									</Label>
									<Input
										defaultValue={prefs.voice_commands.map.close_quote}
										id="map_close_quote"
										name="map_close_quote"
									/>
								</div>
								<div className="grid gap-1">
									<Label className="text-xs" htmlFor="map_backtick">
										Backtick
									</Label>
									<Input
										defaultValue={prefs.voice_commands.map.backtick}
										id="map_backtick"
										name="map_backtick"
									/>
								</div>
								<div className="grid gap-1">
									<Label className="text-xs" htmlFor="map_code_block">
										Code block
									</Label>
									<Input
										defaultValue={prefs.voice_commands.map.code_block}
										id="map_code_block"
										name="map_code_block"
									/>
								</div>
							</div>
						</CardContent>
					</Card>

					<Card>
						<CardHeader>
							<CardTitle>Recording</CardTitle>
						</CardHeader>
						<CardContent className="grid gap-4">
							<div className="grid grid-cols-3 gap-4">
								<div className="grid gap-2">
									<Label htmlFor="chunk_seconds">Chunk (seconds)</Label>
									<Input
										defaultValue={prefs.record.chunk_seconds}
										id="chunk_seconds"
										max="300"
										min="10"
										name="chunk_seconds"
										type="number"
									/>
								</div>
								<div className="grid gap-2">
									<Label htmlFor="max_hours">Max hours</Label>
									<Input
										defaultValue={prefs.record.max_hours}
										id="max_hours"
										max="24"
										min="1"
										name="max_hours"
										type="number"
									/>
								</div>
								<div className="grid gap-2">
									<Label htmlFor="max_file_gb">Max file (GB)</Label>
									<Input
										defaultValue={prefs.record.max_file_gb}
										id="max_file_gb"
										max="16"
										min="1"
										name="max_file_gb"
										type="number"
									/>
								</div>
							</div>
						</CardContent>
					</Card>

					<Button className="w-fit" type="submit">
						Save Settings
					</Button>
				</div>
			</form>
		</div>
	);
}
