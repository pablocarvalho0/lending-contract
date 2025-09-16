/**
 * 👋 Welcome to your Smithery project!
 * To run your server, run "npm run dev"
 *
 * You might find these resources useful:
 *
 * 🧑‍💻 MCP's TypeScript SDK (helps you define your server)
 * https://github.com/modelcontextprotocol/typescript-sdk
 *
 * 📝 smithery.yaml (defines user-level config, like settings or API keys)
 * https://smithery.ai/docs/build/project-config/smithery-yaml
 *
 * 💻 smithery CLI (run "npx @smithery/cli dev" or explore other commands below)
 * https://smithery.ai/docs/concepts/cli
 */

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js"
import { z } from "zod"
import { exec } from "node:child_process"
import { promisify } from "node:util"

const execAsync = promisify(exec)

export const configSchema = z.object({
	debug: z.boolean().default(false).describe("Enable debug logging"),
})

async function runStellar(command: string, debug = false) {
	if (debug) {
		console.log(`[stellar] ${command}`)
	}
	const { stdout, stderr } = await execAsync(command)
	return { stdout, stderr }
}

export default function createServer({
	config,
}: {
	config: z.infer<typeof configSchema>
}) {
	const server = new McpServer({
		name: "Say Hello",
		version: "1.0.0",
	})

	// ============ Tools: Stellar Contract Interactions ============
	const networkSchema = z
		.enum(["testnet", "futurenet", "mainnet", "local"]) // ajuste conforme seu uso
		.default("testnet")

	// __constructor
	server.registerTool(
		"contract_constructor",
		{
			title: "Initialize contract (__constructor)",
			description: "Sets the owner in contract storage",
			inputSchema: {
				contractId: z.string().describe("Contract ID"),
				ownerG: z.string().describe("Owner public address (G...)").min(3),
				sourceS: z.string().describe("Seed (S...) to sign"),
				network: networkSchema,
			},
		},
		async ({ contractId, ownerG, sourceS, network }) => {
			const cmd = `stellar contract invoke --id ${contractId} --network ${network} --source ${sourceS} -- __constructor --owner ${ownerG}`
			const { stdout, stderr } = await runStellar(cmd, config.debug)
			return { content: [{ type: "text", text: stdout || stderr }] }
		},
	)

	// mint (owner-only)
	server.registerTool(
		"nft_mint",
		{
			title: "Mint NFT (owner-only)",
			description: "Mints an NFT to a target address",
			inputSchema: {
				contractId: z.string(),
				toG: z.string().describe("Recipient address (G...)"),
				tokenId: z.number().int().nonnegative(),
				callerG: z.string().describe("Caller address (must be owner)"),
				sourceS: z.string().describe("Seed (S...) of owner"),
				network: networkSchema,
			},
		},
		async ({ contractId, toG, tokenId, callerG, sourceS, network }) => {
			const cmd = `stellar contract invoke --id ${contractId} --network ${network} --source ${sourceS} -- mint --to ${toG} --token_id ${tokenId} --caller ${callerG}`
			const { stdout, stderr } = await runStellar(cmd, config.debug)
			return { content: [{ type: "text", text: stdout || stderr }] }
		},
	)

	// owner_of (read)
	server.registerTool(
		"nft_owner_of",
		{
			title: "Get token owner",
			description: "Returns the owner of a token_id",
			inputSchema: {
				contractId: z.string(),
				tokenId: z.number().int().nonnegative(),
				sourceS: z.string().describe("Seed (S...) of any funded account"),
				network: networkSchema,
			},
		},
		async ({ contractId, tokenId, sourceS, network }) => {
			const cmd = `stellar contract invoke --id ${contractId} --network ${network} --source ${sourceS} -- owner_of --token_id ${tokenId}`
			const { stdout, stderr } = await runStellar(cmd, config.debug)
			return { content: [{ type: "text", text: stdout || stderr }] }
		},
	)

	// create_loan
	server.registerTool(
		"lend_create_loan",
		{
			title: "Create loan using NFT as collateral",
			description: "Borrower must own the NFT",
			inputSchema: {
				contractId: z.string(),
				borrowerG: z.string(),
				tokenId: z.number().int().nonnegative(),
				amount: z.string().describe("i128 as string"),
				interestRate: z.number().int().nonnegative().describe("bps"),
				durationDays: z.number().int().nonnegative(),
				callerG: z.string(),
				sourceS: z.string().describe("Seed (S...) of borrower"),
				network: networkSchema,
			},
		},
		async ({ contractId, borrowerG, tokenId, amount, interestRate, durationDays, callerG, sourceS, network }) => {
			const cmd = `stellar contract invoke --id ${contractId} --network ${network} --source ${sourceS} -- create_loan --borrower ${borrowerG} --token_id ${tokenId} --amount ${amount} --interest_rate ${interestRate} --duration_days ${durationDays} --caller ${callerG}`
			const { stdout, stderr } = await runStellar(cmd, config.debug)
			return { content: [{ type: "text", text: stdout || stderr }] }
		},
	)

	// get_loan_info (read)
	server.registerTool(
		"lend_get_loan_info",
		{
			title: "Get loan info",
			description: "Reads current loan info",
			inputSchema: {
				contractId: z.string(),
				loanId: z.number().int().nonnegative(),
				sourceS: z.string(),
				network: networkSchema,
			},
		},
		async ({ contractId, loanId, sourceS, network }) => {
			const cmd = `stellar contract invoke --id ${contractId} --network ${network} --source ${sourceS} -- get_loan_info --loan_id ${loanId}`
			const { stdout, stderr } = await runStellar(cmd, config.debug)
			return { content: [{ type: "text", text: stdout || stderr }] }
		},
	)

	// is_collateral (read)
	server.registerTool(
		"lend_is_collateral",
		{
			title: "Check if token is collateral",
			description: "Returns true/false",
			inputSchema: {
				contractId: z.string(),
				tokenId: z.number().int().nonnegative(),
				sourceS: z.string(),
				network: networkSchema,
			},
		},
		async ({ contractId, tokenId, sourceS, network }) => {
			const cmd = `stellar contract invoke --id ${contractId} --network ${network} --source ${sourceS} -- is_collateral --token_id ${tokenId}`
			const { stdout, stderr } = await runStellar(cmd, config.debug)
			return { content: [{ type: "text", text: stdout || stderr }] }
		},
	)

	// repay_loan
	server.registerTool(
		"lend_repay_loan",
		{
			title: "Repay loan (borrower-only)",
			description: "Repays amount towards the loan",
			inputSchema: {
				contractId: z.string(),
				loanId: z.number().int().nonnegative(),
				amount: z.string().describe("i128 as string"),
				callerG: z.string(),
				sourceS: z.string().describe("Seed (S...) of borrower"),
				network: networkSchema,
			},
		},
		async ({ contractId, loanId, amount, callerG, sourceS, network }) => {
			const cmd = `stellar contract invoke --id ${contractId} --network ${network} --source ${sourceS} -- repay_loan --loan_id ${loanId} --amount ${amount} --caller ${callerG}`
			const { stdout, stderr } = await runStellar(cmd, config.debug)
			return { content: [{ type: "text", text: stdout || stderr }] }
		},
	)

	// pause / unpause (owner-only)
	server.registerTool(
		"contract_pause",
		{
			title: "Pause contract (owner-only)",
			description: "Activates pause flag",
			inputSchema: {
				contractId: z.string(),
				callerG: z.string(),
				sourceS: z.string().describe("Seed (S...) of owner"),
				network: networkSchema,
			},
		},
		async ({ contractId, callerG, sourceS, network }) => {
			const cmd = `stellar contract invoke --id ${contractId} --network ${network} --source ${sourceS} -- pause --caller ${callerG}`
			const { stdout, stderr } = await runStellar(cmd, config.debug)
			return { content: [{ type: "text", text: stdout || stderr }] }
		},
	)

	server.registerTool(
		"contract_unpause",
		{
			title: "Unpause contract (owner-only)",
			description: "Deactivates pause flag",
			inputSchema: {
				contractId: z.string(),
				callerG: z.string(),
				sourceS: z.string().describe("Seed (S...) of owner"),
				network: networkSchema,
			},
		},
		async ({ contractId, callerG, sourceS, network }) => {
			const cmd = `stellar contract invoke --id ${contractId} --network ${network} --source ${sourceS} -- unpause --caller ${callerG}`
			const { stdout, stderr } = await runStellar(cmd, config.debug)
			return { content: [{ type: "text", text: stdout || stderr }] }
		},
	)

	// ============ Demo hello tool (mantido) ============
	server.registerTool(
		"hello",
		{
			title: "Hello Tool",
			description: "Say hello to someone",
			inputSchema: { name: z.string().describe("Name to greet") },
		},
		async ({ name }) => ({ content: [{ type: "text", text: `Hello, ${name}!` }] }),
	)

	// Demo resource & prompt mantidos
	server.registerResource(
		"hello-world-history",
		"history://hello-world",
		{
			title: "Hello World History",
			description: "The origin story of the famous 'Hello, World' program",
		},
		async uri => ({
			contents: [
				{
					uri: uri.href,
					text:
						'"Hello, World" first appeared in a 1972 Bell Labs memo by Brian Kernighan and later became the iconic first program for beginners in countless languages.',
					mimeType: "text/plain",
				},
			],
		}),
	)

	server.registerPrompt(
		"greet",
		{
			title: "Hello Prompt",
			description: "Say hello to someone",
			argsSchema: {
				name: z.string().describe("Name of the person to greet"),
			},
		},
		async ({ name }) => {
			return {
				messages: [
					{
						role: "user",
						content: { type: "text", text: `Say hello to ${name}` },
					},
				],
			}
		},
	)

	return server.server
}
