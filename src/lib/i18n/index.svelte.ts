import zh from './zh.json';
import en from './en.json';

export type Locale = 'zh' | 'en';
export type Messages = typeof zh;

const messages: Record<Locale, Messages> = { zh, en };

function createI18n() {
	let locale = $state<Locale>('zh');

	function t(key: keyof Messages, fallback?: string): string {
		const msg = messages[locale][key];
		return msg ?? fallback ?? key;
	}

	function setLocale(newLocale: Locale) {
		locale = newLocale;
	}

	return {
		get locale() {
			return locale;
		},
		t,
		setLocale,
	};
}

export const i18n = createI18n();
