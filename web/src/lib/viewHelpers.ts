import type { MessageView } from "@/wasm/logic";

export function messageClass(message: MessageView): string {
  if (message.is_error) {
    return "message message-error";
  }
  if (message.is_success) {
    return "message message-success";
  }
  return "message message-info";
}

export function shouldShowHomeMessage(message: MessageView): boolean {
  return message.visible && !message.is_info;
}
