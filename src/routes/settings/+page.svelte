<script lang="ts">
	import { goto } from '$app/navigation';
	import { open } from '@tauri-apps/plugin-dialog';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import * as Card from '$lib/components/ui/card/index.js';
	import { configStore, type Theme } from '$lib/stores/config.svelte';
	import { i18n, type Locale } from '$lib/i18n';
	import { Folder, Settings, ArrowLeft } from '@lucide/svelte';

	let workDir = $state('');
	let gitUserName = $state('');
	let lang = $state<Locale>('zh');
	let theme = $state<Theme>('system');
	let aiProvider = $state<string>('');
	let aiApiKey = $state('');
	let aiBaseUrl = $state('');
	let aiModel = $state('');

	async function init() {
		await configStore.refresh();
		workDir = configStore.configs.work_dir ?? '';
		gitUserName = configStore.configs.git_user_name ?? '';
		lang = configStore.configs.lang ?? 'zh';
		theme = configStore.configs.theme ?? 'system';
		aiProvider = configStore.configs.ai_provider ?? '';
		aiApiKey = configStore.configs.ai_api_key ?? '';
		aiBaseUrl = configStore.configs.ai_base_url ?? '';
		aiModel = configStore.configs.ai_model ?? '';
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
			ai_provider: aiProvider || undefined,
			ai_api_key: aiApiKey.trim() || undefined,
			ai_base_url: aiProvider === 'custom' ? aiBaseUrl.trim() || undefined : undefined,
			ai_model: aiModel.trim() || undefined,
		});
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
	const selectedProviderLabel = $derived(providerItems.find((i) => i.value === aiProvider)?.label ?? '');
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
						<Select.Root type="single" bind:value={aiProvider}>
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
							<Label for="ai-model">{i18n.t('config.ai.model')}</Label>
							<Input
								id="ai-model"
								bind:value={aiModel}
								placeholder={i18n.t('common.optional')}
							/>
						</div>
					{/if}
				</Card.Content>
			</Card.Root>
		</div>
	</div>
</div>
