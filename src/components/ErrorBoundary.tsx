import { Component, ErrorInfo, ReactNode } from "react";
import { ErrorState } from "./ErrorState";
import { ERROR_MESSAGES } from "../constants/errorMessages";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error("Uncaught error:", error, errorInfo);
  }

  handleRetry = () => {
    this.setState({ hasError: false, error: null });
    window.location.reload();
  };

  // Traduz mensagens técnicas para suas constantes
  getFriendlyMessage = (originalMessage: string) => {
    if (originalMessage.includes("Rendered fewer hooks")) {
      return ERROR_MESSAGES.RENDER_HOOKS_ERROR;
    }

    if (originalMessage.includes("is not a function")) {
      return ERROR_MESSAGES.RENDER_FUNCTION_ERROR;
    }

    if (originalMessage.includes("is not defined")) {
      return ERROR_MESSAGES.RENDER_VARIABLE_ERROR;
    }

    // Fallback genérico se não reconhecer o erro
    return originalMessage || ERROR_MESSAGES.RENDER_GENERIC_ERROR;
  };

  render() {
    if (this.state.hasError) {
      const originalMsg = this.state.error?.message || "";
      const friendlyMsg = this.getFriendlyMessage(originalMsg);

      return (
        <div className="h-full flex flex-col items-center justify-center p-8 animate-in fade-in">
          <ErrorState
            type="generic"
            message={friendlyMsg}
            onRetry={this.handleRetry} // Passa a função, o ErrorState cria o botão
          />
        </div>
      );
    }

    return this.props.children;
  }
}
