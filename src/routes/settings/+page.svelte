<script lang="ts">
	import { goto } from '$app/navigation';
	import { open } from '@tauri-apps/plugin-dialog';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import * as Card from '$lib/components/ui/card/index.js';
	import { configStore, type Theme } from '$lib/stores/config.svelte';
	import { toastStore } from '$lib/stores/toast.svelte';
	import { i18n, type Locale } from '$lib/i18n';
	import { Folder, Settings, ArrowLeft, RefreshCw } from '@lucide/svelte';

	let workDir = $state('');
	let gitUserName = $state('');
	let lang = $state<Locale>('zh');
	let theme = $state<Theme>('system');
	let weekStartDay = $state<number>(1);
	let aiProvider = $state<string>('');
	let aiApiKey = $state('');
	let aiBaseUrl = $state('');
	let aiModel = $state('');
	let aiTemplate = $state('');
	let fetchedModels = $state<{ id: string; label: string }[]>([]);
	let fetchingModels = $state(false);

	const DEFAULT_TEMPLATE_ZH = '# 年-月-日 工作日报\n## 今日工作概览\n\n## 今日完成任务\n\n## 明日工作计划';
	const DEFAULT_TEMPLATE_EN = "# YYYY-MM-DD Daily Report\n## Today's Overview\n\n## Completed Tasks\n\n## Tomorrow's Plan";

	async function init() {
		await configStore.refresh();
		workDir = configStore.configs.work_dir ?? '';
		gitUserName = configStore.configs.git_user_name ?? '';
		lang = configStore.configs.lang ?? 'zh';
		theme = configStore.configs.theme ?? 'system';
		weekStartDay = configStore.configs.week_start_day ?? 1;
		aiProvider = configStore.configs.ai_provider ?? '';
		aiApiKey = configStore.configs.ai_api_key ?? '';
		aiBaseUrl = configStore.configs.ai_base_url ?? '';
		aiModel = configStore.configs.ai_model ?? '';
		const defaultTpl = lang === 'en' ? DEFAULT_TEMPLATE_EN : DEFAULT_TEMPLATE_ZH;
		aiTemplate = configStore.configs.ai_template ?? defaultTpl;
	}

	init();

	async function pickWorkDir() {
		const selected = await open({
			directory: true,
			multiple: false,
		});
		if (selected && typeof selected === 'string') {
			workDir = selected;
		}
	}

	async function handleSave() {
		await configStore.save({
			work_dir: workDir.trim() || undefined,
			git_user_name: gitUserName.trim() || undefined,
			lang,
			theme,
			week_start_day: weekStartDay,
			ai_provider: aiProvider || undefined,
			ai_api_key: aiApiKey.trim() || undefined,
			ai_base_url: aiProvider === 'custom' ? aiBaseUrl.trim() || undefined : undefined,
			ai_model: aiModel.trim() || undefined,
			ai_template: aiTemplate.trim() || undefined,
		});
		toastStore.show(i18n.t('settings.saveSuccess'));
	}

	const langItems = $derived([
		{ value: 'zh' as Locale, label: i18n.t('config.lang.zh') },
		{ value: 'en' as Locale, label: i18n.t('config.lang.en') },
	]);

	const themeItems = $derived([
		{ value: 'light' as Theme, label: i18n.t('config.theme.light') },
		{ value: 'dark' as Theme, label: i18n.t('config.theme.dark') },
		{ value: 'system' as Theme, label: i18n.t('config.theme.system') },
	]);

	const weekStartDayItems = $derived([
		{ value: 1, label: i18n.t('config.weekStartDay.1') },
		{ value: 2, label: i18n.t('config.weekStartDay.2') },
		{ value: 3, label: i18n.t('config.weekStartDay.3') },
		{ value: 4, label: i18n.t('config.weekStartDay.4') },
		{ value: 5, label: i18n.t('config.weekStartDay.5') },
		{ value: 6, label: i18n.t('config.weekStartDay.6') },
		{ value: 7, label: i18n.t('config.weekStartDay.7') },
	]);

	const providerItems = $derived([
		{ value: '', label: i18n.t('common.optional') },
		{ value: 'openai', label: i18n.t('config.ai.provider.openai') },
		{ value: 'anthropic', label: i18n.t('config.ai.provider.anthropic') },
		{ value: 'gemini', label: i18n.t('config.ai.provider.gemini') },
		{ value: 'deepseek', label: i18n.t('config.ai.provider.deepseek') },
		{ value: 'custom', label: i18n.t('config.ai.provider.custom') },
	]);

	const selectedLangLabel = $derived(langItems.find((i) => i.value === lang)?.label ?? '');
	const selectedThemeLabel = $derived(themeItems.find((i) => i.value === theme)?.label ?? '');
	const selectedWeekStartDayLabel = $derived(weekStartDayItems.find((i) => i.value === weekStartDay)?.label ?? '');
	const selectedProviderLabel = $derived(providerItems.find((i) => i.value === aiProvider)?.label ?? '');

	const staticModelMap: Record<string, { id: string; label: string }[]> = {
		anthropic: [
			{ id: 'claude-3-5-sonnet-latest', label: 'Claude 3.5 Sonnet' },
			{ id: 'claude-3-opus-latest', label: 'Claude 3 Opus' },
			{ id: 'claude-3-5-haiku-latest', label: 'Claude 3.5 Haiku' },
		],
		gemini: [
			{ id: 'gemini-1.5-pro-latest', label: 'Gemini 1.5 Pro' },
			{ id: 'gemini-1.5-flash-latest', label: 'Gemini 1.5 Flash' },
			{ id: 'gemini-1.0-pro', label: 'Gemini 1.0 Pro' },
		],
	};

	const providerBaseUrls: Record<string, string> = {
		openai: 'https://api.openai.com/v1',
		deepseek: 'https://api.deepseek.com/v1',
	};

	const canFetchModels = $derived(
		aiProvider === 'openai' || aiProvider === 'deepseek' || aiProvider === 'custom'
	);

	const modelItems = $derived(
		fetchedModels.length > 0
			? fetchedModels
			: (staticModelMap[aiProvider] ?? [])
	);

	const selectedModelLabel = $derived(modelItems.find((i) => i.id === aiModel)?.label ?? '');

	async function fetchModels() {
		if (!aiApiKey.trim()) return;
		let baseUrl = providerBaseUrls[aiProvider];
		if (aiProvider === 'custom') {
			baseUrl = aiBaseUrl.trim();
		}
		if (!baseUrl) return;

		fetchingModels = true;
		try {
			const res = await fetch(`${baseUrl}/models`, {
				headers: {
					Authorization: `Bearer ${aiApiKey.trim()}`,
				},
			});
			if (!res.ok) {
				console.error('failed to fetch models:', res.status, await res.text());
				return;
			}
			const data = await res.json();
			const list: { id: string; label: string }[] = (data.data ?? [])
				.filter((m: any) => m.id && typeof m.id === 'string')
				.map((m: any) => ({ id: m.id, label: m.id }));
			fetchedModels = list;
		} catch (e) {
			console.error('failed to fetch models:', e);
		} finally {
			fetchingModels = false;
		}
	}

	function handleProviderChange(newProvider: string) {
		aiProvider = newProvider;
		fetchedModels = [];
		aiModel = '';
	}
</script>

<div class="flex flex-col h-screen w-full overflow-hidden bg-background">
	<!-- Top bar -->
	<div class="flex items-center justify-between px-4 h-12 border-b shrink-0 bg-background">
		<div class="flex items-center gap-3">
			<Button variant="ghost" size="icon" class="h-8 w-8" onclick={() => goto('/')} title={i18n.t('settings.back')}>
				<ArrowLeft class="h-4 w-4" />
			</Button>
			<Settings class="h-4 w-4 text-muted-foreground" />
			<span class="text-sm font-semibold">{i18n.t('settings.title')}</span>
		</div>
		<Button size="sm" onclick={handleSave}>{i18n.t('common.save')}</Button>
	</div>

	<!-- Settings content with scroll -->
	<div class="flex-1 overflow-auto">
		<div class="p-6 max-w-2xl mx-auto space-y-6 pb-12">
			<!-- General Settings -->
			<Card.Root>
				<Card.Header>
					<Card.Title>{i18n.t('settings.general')}</Card.Title>
				</Card.Header>
				<Card.Content class="space-y-5">
					<div class="grid gap-2">
						<Label for="work-dir">{i18n.t('config.workDir')}</Label>
						<div class="flex gap-2">
							<Input
								id="work-dir"
								bind:value={workDir}
								placeholder={i18n.t('config.workDir')}
								class="flex-1"
								readonly
							/>
							<Button variant="outline" size="icon" onclick={pickWorkDir} title="选择文件夹">
								<Folder class="h-4 w-4" />
							</Button>
						</div>
					</div>

					<div class="grid gap-2">
						<Label for="git-user-name">{i18n.t('config.gitUserName')}</Label>
						<Input
							id="git-user-name"
							bind:value={gitUserName}
							placeholder={i18n.t('config.gitUserName')}
						/>
					</div>

					<div class="grid gap-2">
						<Label>{i18n.t('config.language')}</Label>
						<Select.Root type="single" bind:value={lang}>
							<Select.Trigger class="w-full">
								{selectedLangLabel}
							</Select.Trigger>
							<Select.Content>
								{#each langItems as item (item.value)}
									<Select.Item value={item.value} label={item.label}>
										{item.label}
									</Select.Item>
								{/each}
							</Select.Content>
						</Select.Root>
					</div>

					<div class="grid gap-2">
						<Label>{i18n.t('config.theme')}</Label>
						<Select.Root type="single" bind:value={theme}>
							<Select.Trigger class="w-full">
								{selectedThemeLabel}
							</Select.Trigger>
							<Select.Content>
								{#each themeItems as item (item.value)}
									<Select.Item value={item.value} label={item.label}>
										{item.label}
									</Select.Item>
								{/each}
							</Select.Content>
						</Select.Root>
					</div>

					<div class="grid gap-2">
						<Label>{i18n.t('config.weekStartDay')}</Label>
						<Select.Root type="single" bind:value={weekStartDay}>
							<Select.Trigger class="w-full">
								{selectedWeekStartDayLabel}
							</Select.Trigger>
							<Select.Content>
								{#each weekStartDayItems as item (item.value)}
									<Select.Item value={item.value} label={item.label}>
										{item.label}
									</Select.Item>
								{/each}
							</Select.Content>
						</Select.Root>
					</div>
				</Card.Content>
			</Card.Root>

			<!-- AI Settings -->
			<Card.Root>
				<Card.Header>
					<Card.Title>{i18n.t('settings.ai')}</Card.Title>
				</Card.Header>
				<Card.Content class="space-y-5">
					<div class="grid gap-2">
						<Label>{i18n.t('config.ai.provider')}</Label>
						<Select.Root type="single" value={aiProvider} onValueChange={handleProviderChange}>
							<Select.Trigger class="w-full">
								{selectedProviderLabel}
							</Select.Trigger>
							<Select.Content>
								{#each providerItems as item (item.value)}
									<Select.Item value={item.value} label={item.label}>
										{item.label}
									</Select.Item>
								{/each}
							</Select.Content>
						</Select.Root>
					</div>

					{#if aiProvider}
						<div class="grid gap-2">
							<Label for="ai-api-key">{i18n.t('config.ai.apiKey')}</Label>
							<Input
								id="ai-api-key"
								type="password"
								bind:value={aiApiKey}
								placeholder={i18n.t('common.optional')}
							/>
						</div>

						{#if aiProvider === 'custom'}
							<div class="grid gap-2">
								<Label for="ai-base-url">{i18n.t('config.ai.baseUrl')}</Label>
								<Input
									id="ai-base-url"
									bind:value={aiBaseUrl}
									placeholder={i18n.t('config.ai.baseUrlHint')}
								/>
							</div>
						{/if}

						<div class="grid gap-2">
							<div class="flex items-center justify-between">
								<Label for="ai-model">{i18n.t('config.ai.model')}</Label>
								{#if canFetchModels}
									<Button
										variant="ghost"
										size="sm"
										class="h-6 px-2 text-xs"
										onclick={fetchModels}
										disabled={fetchingModels || !aiApiKey.trim()}
									>
										<RefreshCw class="h-3 w-3 mr-1" />
										{fetchingModels ? i18n.t('config.ai.fetching') : i18n.t('config.ai.fetchModels')}
									</Button>
								{/if}
							</div>
							{#if aiProvider === 'custom'}
								<Input
									id="ai-model"
									bind:value={aiModel}
									placeholder={i18n.t('common.optional')}
								/>
							{:else}
								<Select.Root type="single" bind:value={aiModel}>
									<Select.Trigger class="w-full">
										{selectedModelLabel || i18n.t('common.optional')}
									</Select.Trigger>
									<Select.Content>
										{#each modelItems as item (item.id)}
											<Select.Item value={item.id} label={item.label}>
												{item.label}
											</Select.Item>
										{/each}
									</Select.Content>
								</Select.Root>
							{/if}
						</div>

						<div class="grid gap-2">
							<Label for="ai-template">{i18n.t('config.ai.template')}</Label>
							<Textarea
								id="ai-template"
								bind:value={aiTemplate}
								placeholder={i18n.t('config.ai.templateHint')}
								rows={10}
								class="font-mono text-xs"
							/>
							<div class="text-xs text-muted-foreground">
								{i18n.t('config.ai.templateVars')}
							</div>
						</div>
					{/if}
				</Card.Content>
			</Card.Root>
		</div>
	</div>
</div>
