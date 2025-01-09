import { type ChildProcess, spawn } from "child_process";
import { EventEmitter } from "events";

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
        this.process = spawn(this.serverPath, {
          stdio: ["pipe", "pipe", "pipe"],
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
    const header = `Content-Length: ${Buffer.byteLength(content, "utf8")}\r\n\r\n`;
    this.process?.stdin?.write(header + content);
  }

  private handleData(data: Buffer): void {
    this.messageBuffer += data.toString();

    while (true) {
      const headerMatch = this.messageBuffer.match(
        /Content-Length: (\d+)\r\n\r\n/,
      );
      if (!headerMatch) {
        break;
      }

      const contentLength = Number.parseInt(headerMatch[1], 10);
      const headerEnd = headerMatch.index! + headerMatch[0].length;

      if (this.messageBuffer.length < headerEnd + contentLength) {
        break;
      }

      const messageStr = this.messageBuffer.substr(headerEnd, contentLength);
      const message: Message = JSON.parse(messageStr);
      this.messageBuffer = this.messageBuffer.substr(headerEnd + contentLength);
      this.handleMessage(message);
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
