import { createFileRoute } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import { api, type Preferences } from "../lib/api";

export const Route = createFileRoute("/settings")({
	component: SettingsComponent,
});

function SettingsComponent() {
	const [prefs, setPrefs] = useState<Preferences | null>(null);
	const [loading, setLoading] = useState(true);
	const [saving, setSaving] = useState(false);

	useEffect(() => {
		api
			.getPreferences()
			.then(setPrefs)
			.catch(() => {
				toast.error("Failed to load preferences");
			})
			.finally(() => setLoading(false));
	}, []);

	const handleSave = async () => {
		if (!prefs) {
			return;
		}
		setSaving(true);
		try {
			await api.updatePreferences(prefs);
			toast.success("Preferences saved");
		} catch {
			toast.error("Failed to save preferences");
		} finally {
			setSaving(false);
		}
	};

	if (loading) {
		return (
			<div className="container mx-auto max-w-3xl px-4 py-2">
				<p>Loading...</p>
			</div>
		);
	}

	if (!prefs) {
		return (
			<div className="container mx-auto max-w-3xl px-4 py-2">
				<p>Failed to load preferences</p>
			</div>
		);
	}

	return (
		<div className="container mx-auto max-w-3xl px-4 py-2">
			<h1 className="mb-6 font-bold text-2xl">Settings</h1>

			<div className="grid gap-6">
				<section className="rounded-lg border p-4">
					<h2 className="mb-4 font-medium">Hotkeys</h2>
					<div className="space-y-2">
						<label className="flex items-center gap-2">
							<input
								checked={prefs.hotkeys.left_chord}
								onChange={(e) =>
									setPrefs({
										...prefs,
										hotkeys: {
											...prefs.hotkeys,
											left_chord: e.target.checked,
										},
									})
								}
								type="checkbox"
							/>
							Left Cmd+Opt
						</label>
						<label className="flex items-center gap-2">
							<input
								checked={prefs.hotkeys.right_chord}
								onChange={(e) =>
									setPrefs({
										...prefs,
										hotkeys: {
											...prefs.hotkeys,
											right_chord: e.target.checked,
										},
									})
								}
								type="checkbox"
							/>
							Right Cmd+Opt
						</label>
					</div>
				</section>

				<section className="rounded-lg border p-4">
					<h2 className="mb-4 font-medium">Activation</h2>
					<div className="space-y-4">
						<label className="block">
							<span className="mb-1 block text-sm">Mode</span>
							<select
								className="w-full rounded border p-2"
								onChange={(e) =>
									setPrefs({
										...prefs,
										mode: e.target.value as "hold" | "toggle",
									})
								}
								value={prefs.mode}
							>
								<option value="hold">Hold</option>
								<option value="toggle">Toggle</option>
							</select>
						</label>
						<label className="block">
							<span className="mb-1 block text-sm">
								Silence before stopping (seconds)
							</span>
							<input
								className="w-full rounded border p-2"
								onChange={(e) =>
									setPrefs({
										...prefs,
										silence_seconds: Number.parseFloat(e.target.value),
									})
								}
								step="0.5"
								type="number"
								value={prefs.silence_seconds}
							/>
						</label>
						<label className="block">
							<span className="mb-1 block text-sm">Silence sensitivity</span>
							<select
								className="w-full rounded border p-2"
								onChange={(e) =>
									setPrefs({
										...prefs,
										silence_rms: e.target.value as "low" | "medium" | "high",
									})
								}
								value={prefs.silence_rms}
							>
								<option value="low">Low</option>
								<option value="medium">Medium</option>
								<option value="high">High</option>
							</select>
						</label>
					</div>
				</section>

				<section className="rounded-lg border p-4">
					<h2 className="mb-4 font-medium">Model</h2>
					<label className="block">
						<span className="mb-1 block text-sm">Profile</span>
						<select
							className="w-full rounded border p-2"
							onChange={(e) =>
								setPrefs({
									...prefs,
									model_profile: e.target.value as
										| "small.en"
										| "multilingual-small"
										| "multilingual-medium",
								})
							}
							value={prefs.model_profile}
						>
							<option value="small.en">English (Small)</option>
							<option value="multilingual-small">Multilingual (Small)</option>
							<option value="multilingual-medium">Multilingual (Medium)</option>
						</select>
					</label>
					<label className="mt-4 block">
						<input
							checked={prefs.translate_to_english}
							onChange={(e) =>
								setPrefs({
									...prefs,
									translate_to_english: e.target.checked,
								})
							}
							type="checkbox"
						/>
						<span className="ml-2">Translate to English</span>
					</label>
				</section>

				<section className="rounded-lg border p-4">
					<h2 className="mb-4 font-medium">Typing</h2>
					<div className="space-y-4">
						<label className="flex items-center gap-2">
							<input
								checked={prefs.typing.newline_at_end}
								onChange={(e) =>
									setPrefs({
										...prefs,
										typing: {
											...prefs.typing,
											newline_at_end: e.target.checked,
										},
									})
								}
								type="checkbox"
							/>
							Add newline at end
						</label>
						<label className="block">
							<span className="mb-1 block text-sm">Throttle (ms)</span>
							<input
								className="w-full rounded border p-2"
								min="0"
								onChange={(e) =>
									setPrefs({
										...prefs,
										typing: {
											...prefs.typing,
											throttle_ms: Number.parseInt(e.target.value, 10),
										},
									})
								}
								type="number"
								value={prefs.typing.throttle_ms}
							/>
						</label>
					</div>
				</section>

				<section className="rounded-lg border p-4">
					<h2 className="mb-4 font-medium">Voice Commands</h2>
					<label className="mb-4 block">
						<input
							checked={prefs.voice_commands.enabled}
							onChange={(e) =>
								setPrefs({
									...prefs,
									voice_commands: {
										...prefs.voice_commands,
										enabled: e.target.checked,
									},
								})
							}
							type="checkbox"
						/>
						<span className="ml-2">Enable voice commands</span>
					</label>
					{prefs.voice_commands.enabled && (
						<div className="grid grid-cols-2 gap-4">
							{(
								Object.entries(prefs.voice_commands.map) as [string, string][]
							).map(([key, value]) => (
								<label className="block" key={key}>
									<span className="mb-1 block text-sm capitalize">
										{key.replace(/_/g, " ")}
									</span>
									<input
										className="w-full rounded border p-2"
										onChange={(e) =>
											setPrefs({
												...prefs,
												voice_commands: {
													...prefs.voice_commands,
													map: {
														...prefs.voice_commands.map,
														[key]: e.target.value,
													},
												},
											})
										}
										type="text"
										value={value}
									/>
								</label>
							))}
						</div>
					)}
				</section>

				<section className="rounded-lg border p-4">
					<h2 className="mb-4 font-medium">Record Mode</h2>
					<div className="space-y-4">
						<label className="block">
							<span className="mb-1 block text-sm">
								Chunk duration (seconds)
							</span>
							<input
								className="w-full rounded border p-2"
								min="30"
								onChange={(e) =>
									setPrefs({
										...prefs,
										record: {
											...prefs.record,
											chunk_seconds: Number.parseInt(e.target.value, 10),
										},
									})
								}
								type="number"
								value={prefs.record.chunk_seconds}
							/>
						</label>
						<label className="block">
							<span className="mb-1 block text-sm">Max hours</span>
							<input
								className="w-full rounded border p-2"
								min="1"
								onChange={(e) =>
									setPrefs({
										...prefs,
										record: {
											...prefs.record,
											max_hours: Number.parseInt(e.target.value, 10),
										},
									})
								}
								type="number"
								value={prefs.record.max_hours}
							/>
						</label>
						<label className="block">
							<span className="mb-1 block text-sm">Max file size (GB)</span>
							<input
								className="w-full rounded border p-2"
								min="1"
								onChange={(e) =>
									setPrefs({
										...prefs,
										record: {
											...prefs.record,
											max_file_gb: Number.parseInt(e.target.value, 10),
										},
									})
								}
								type="number"
								value={prefs.record.max_file_gb}
							/>
						</label>
					</div>
				</section>

				<button
					className="rounded bg-primary px-4 py-2 text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
					disabled={saving}
					onClick={handleSave}
					type="button"
				>
					{saving ? "Saving..." : "Save Preferences"}
				</button>
			</div>
		</div>
	);
}
