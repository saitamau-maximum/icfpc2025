const RANKING_API_URL = 'https://31pwr5t6ij.execute-api.eu-west-2.amazonaws.com/leaderboard/global';
const CONTEST_START_AT = new Date('2025-09-05T12:00:00Z');
const CONTEST_END_AT = new Date('2025-09-08T12:00:00Z');

const isContestPeriod = (date: Date) => date >= CONTEST_START_AT && date <= CONTEST_END_AT;

interface ResultItem {
	teamName: string;
	teamPl: string;
	score: number;
}

type APIResponse = ResultItem[];

const responseToMarkdown = (results: APIResponse) => {
	let markdown = '';
	let rank = 1;
	let prevScore = results[0].score;
	const RANK_MAX_LENGTH = 5;
	const TEAM_NAME_MAX_LENGTH = 60;
	const SCORE_MAX_LENGTH = 5;
	markdown += '```\n';
	markdown += `| ${'Rank'.padEnd(RANK_MAX_LENGTH)} | ${'Team Name'.padEnd(TEAM_NAME_MAX_LENGTH)} | ${'Score'.padEnd(SCORE_MAX_LENGTH)} |\n`;
	markdown += '| ' + '-'.repeat(RANK_MAX_LENGTH) + ' | ' + '-'.repeat(TEAM_NAME_MAX_LENGTH) + ' | ' + '-'.repeat(SCORE_MAX_LENGTH) + ' |\n';
	let idx = 0;
	for (const result of results) {
		if (result.score !== prevScore) {
			rank++;
			prevScore = result.score;
		}
		if (result.teamName === 'Maximum' || idx < 10) {
			markdown += `| ${rank.toString().padEnd(RANK_MAX_LENGTH)} | ${result.teamName.padEnd(TEAM_NAME_MAX_LENGTH)} | ${result.score
				.toString()
				.padEnd(SCORE_MAX_LENGTH)} |\n`;
		}
		idx++;
	}
	markdown += '```\n';
	return markdown;
};

export default {
	async scheduled(_event, env, _ctx): Promise<void> {
		if (!isContestPeriod(new Date())) {
			console.log('not in contest period');
			return;
		}
		let resp = await fetch(RANKING_API_URL);
		const results = (await resp.json()) as APIResponse;
		await fetch(env.DISCORD_WEBHOOK_URL, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json',
			},
			body: JSON.stringify({
				content: responseToMarkdown(results),
			}),
		});
	},
} satisfies ExportedHandler<Env>;
