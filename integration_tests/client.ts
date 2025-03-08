import { type ChildProcess, spawn } from "node:child_process";
import { EventEmitter } from "node:events";

interface Message {
	jsonrpc: "2.0";
	id?: number;
	method?: string;
	params?: any;
	result?: any;
	error?: {
		code: number;
		message: string;
		data?: any;
	};
}

export class LSPTestClient extends EventEmitter {
	private process: ChildProcess | null = null;
	private messageBuffer = "";
	private nextMessageId = 1;
	private pendingRequests: Map<
		number,
		{ resolve: Function; reject: Function }
	> = new Map();
	private nextMessagePromise: { resolve: Function; reject: Function } | null =
		null;

	constructor(private serverPath: string) {
		super();
	}

	async start(): Promise<void> {
		return new Promise((resolve, reject) => {
			try {
				this.process = spawn(this.serverPath, ["serve"], {
					stdio: ["pipe", "pipe", "pipe"],
					env: { RUST_BACKTRACE: "1", RUST_LOG: "debug" },
				});

				this.process.stdout?.on("data", (data) => this.handleData(data));
				this.process.stderr?.on("data", (data) =>
					console.error(`Server Log: ${data}`),
				);

				this.process.on("error", (error) => {
					console.error("Failed to start server:", error);
					reject(error);
				});

				this.process.on("exit", (code) => {
					if (code !== 0) {
						console.error(`Server exited with code ${code}`);
					}
				});

				// Send initialize request
				this.sendRequest("initialize", {
					capabilities: {},
					processId: process.pid,
					rootUri: null,
					workspaceFolders: null,
				})
					.then(() => {
						this.sendNotification("initialized", {});
						resolve();
					})
					.catch(reject);
			} catch (error) {
				reject(error);
			}
		});
	}

	async stop(): Promise<void> {
		if (!this.process) {
			return;
		}

		try {
			await this.sendNotification("exit");
			this.process.kill();
			this.process = null;
			this.messageBuffer = "";
			this.pendingRequests.clear();
			if (this.nextMessagePromise) {
				this.nextMessagePromise.reject(new Error("Client stopped"));
				this.nextMessagePromise = null;
			}
		} catch (error) {
			console.error("Error stopping server:", error);
			throw error;
		}
	}

	async sendRequest<T>(method: string, params?: any): Promise<T> {
		if (!this.process) {
			throw new Error("Client not started");
		}

		const id = this.nextMessageId++;
		const message: Message = {
			jsonrpc: "2.0",
			id,
			method,
			params,
		};

		return new Promise((resolve, reject) => {
			this.pendingRequests.set(id, { resolve, reject });
			this.send(message);
		});
	}

	async sendNotification(method: string, params?: any): Promise<void> {
		if (!this.process) {
			throw new Error("Client not started");
		}

		const message: Message = {
			jsonrpc: "2.0",
			method,
			params,
		};

		this.send(message);
	}

	waitForNextMessage(): Promise<Message> {
		return new Promise((resolve, reject) => {
			this.nextMessagePromise = { resolve, reject };
		});
	}

	private send(message: Message): void {
		const content = JSON.stringify(message);
		// Calculate the byte length correctly for UTF-8
		const contentLength = Buffer.byteLength(content, "utf8");
		const header = `Content-Length: ${contentLength}\r\n\r\n`;

		// Ensure the process stdin exists
		if (!this.process?.stdin) {
			throw new Error("Process stdin is not available");
		}

		// Send as a single write to avoid potential split issues
		const fullMessage = header + content;

		// Handle potential backpressure
		const success = this.process.stdin.write(fullMessage, "utf8");
		if (!success) {
			// If the stream signals backpressure, wait for the 'drain' event
			this.process.stdin.once("drain", () => {
				console.log("Stdin drain event occurred");
			});
		}
	}

	private handleData(data: Buffer): void {
		// Append new data
		this.messageBuffer += data.toString("utf8");

		// Process all complete messages in the buffer
		while (true) {
			// Find the header
			const headerMatch = this.messageBuffer.match(
				/Content-Length: (\d+)\r\n\r\n/,
			);
			if (!headerMatch) break;

			const contentLength = Number.parseInt(headerMatch[1], 10);
			const headerEnd = headerMatch.index! + headerMatch[0].length;

			// Convert the entire buffer to bytes for accurate measurement
			const messageBufferBytes = Buffer.from(this.messageBuffer, "utf8");
			const headerEndBytes = Buffer.from(
				this.messageBuffer.substring(0, headerEnd),
				"utf8",
			).length;

			// Check if we have the full message
			if (messageBufferBytes.length < headerEndBytes + contentLength) {
				break; // Need more data
			}

			// Extract the content bytes and convert to string
			const contentBytes = messageBufferBytes.slice(
				headerEndBytes,
				headerEndBytes + contentLength,
			);
			const contentString = contentBytes.toString("utf8");

			try {
				// Parse the message
				const message = JSON.parse(contentString);

				// Calculate where this message ends in the original string
				// This is tricky with multi-byte chars, so use the content string length
				const bufferSegmentToHeaderEnd = this.messageBuffer.substring(
					0,
					headerEnd,
				);
				const bytesToHeaderEnd = Buffer.from(
					bufferSegmentToHeaderEnd,
					"utf8",
				).length;
				const contentStringBytes = Buffer.from(contentString, "utf8").length;

				// Find where to cut the buffer for the next iteration
				let cutPoint = headerEnd;
				let measuredBytes = 0;

				while (
					measuredBytes < contentStringBytes &&
					cutPoint < this.messageBuffer.length
				) {
					const char = this.messageBuffer[cutPoint];
					measuredBytes += Buffer.from(char, "utf8").length;
					cutPoint++;
				}

				// Update buffer and process message
				this.messageBuffer = this.messageBuffer.substring(cutPoint);
				this.handleMessage(message);
			} catch (e) {
				console.error("Error handling message:", e);
				// Skip this message on error
				this.messageBuffer = this.messageBuffer.substring(headerEnd);
				break;
			}
		}
	}

	private handleMessage(message: Message): void {
		// Handle any waiting next message promise
		if (this.nextMessagePromise) {
			const { resolve } = this.nextMessagePromise;
			this.nextMessagePromise = null;
			resolve(message);
		}

		// Handle response
		if (message.id) {
			const pending = this.pendingRequests.get(message.id);
			if (pending) {
				this.pendingRequests.delete(message.id);
				if ("error" in message) {
					pending.reject(message.error);
				} else {
					pending.resolve(message.result);
				}
			}
		}
		// Handle notification
		else if (message.method) {
			this.emit(message.method, message.params);
		}
	}
}
