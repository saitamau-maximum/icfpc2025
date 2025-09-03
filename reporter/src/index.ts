import { HTMLElement, parse } from 'node-html-parser';

const RANKING_PAGE_URL = 'https://icfpcontest2025.github.io/aedificium.html';
const CONTEST_START_AT = new Date('2025-09-05T12:00:00Z');
const CONTEST_END_AT = new Date('2025-09-08T12:00:00Z');

const isContestPeriod = (date: Date) => date >= CONTEST_START_AT && date <= CONTEST_END_AT;
const is0Minute = (date: Date) => date.getMinutes() === 0;

const isMaximum = (text: string) => text === 'Maximum';

const isMaximumRow = (row: HTMLElement) => row.querySelectorAll('td').some((cell) => isMaximum(cell.text));

const tableToMarkdown = (table: HTMLElement) => {
	let markdown = '';
	const header = table.querySelector('thead');
	if (!header) {
		console.error('Header not found');
		return;
	}
	const headerCells = header.querySelectorAll('th');
	// 各列で最大の文字数を数える
	const maxLengths = headerCells.map((cell) => cell.text.length);
	const rows = table.querySelectorAll('tbody tr');
	let maximumRank;
	rows.forEach((row) => {
		if (isMaximumRow(row)) {
			maximumRank = row.querySelectorAll('td')[0].text;
		}
		const cells = row.querySelectorAll('td');
		cells.forEach((cell, index) => {
			maxLengths[index] = Math.max(maxLengths[index], cell.text.length);
		});
	});
	if (maximumRank) {
		markdown += `Maximum is **${maximumRank}th**\n`;
	}
	markdown += `${RANKING_PAGE_URL}\n`;
	markdown += '```markdown\n';
	markdown += `| ${headerCells.map((cell, index) => cell.text.padEnd(maxLengths[index])).join(' | ')} |`;
	markdown += '\n';
	markdown += `| ${headerCells.map((_, index) => '-'.repeat(maxLengths[index])).join(' | ')} |`;
	markdown += '\n';
	let isMaximumIncluded = false;
	// トップ10 + Maximum を表示
	rows.forEach((row, index) => {
		if (isMaximumRow(row)) isMaximumIncluded = true;
		if (isMaximumIncluded && index > 10) return;
		const cells = row.querySelectorAll('td');
		markdown += `| ${cells.map((cell, index) => cell.text.padEnd(maxLengths[index])).join(' | ')} |`;
		markdown += '\n';
	});
	markdown += '```\n';
	return markdown;
};

export default {
	async scheduled(_event, env, _ctx): Promise<void> {
		if (!isContestPeriod(new Date())) {
			console.log('not in contest period');
			return;
		}
		if (!is0Minute(new Date())) {
			console.log('not 0 minute');
			return;
		}
		let resp = await fetch(RANKING_PAGE_URL);
		const root = parse(await resp.text());
		const table = root.querySelector('table');
		if (!table) {
			console.error('Table not found');
			return;
		}
		await fetch(env.DISCORD_WEBHOOK_URL, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json',
			},
			body: JSON.stringify({
				content: tableToMarkdown(table),
			}),
		});
	},
} satisfies ExportedHandler<Env>;
