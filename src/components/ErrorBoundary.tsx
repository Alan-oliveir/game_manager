import { Component, ErrorInfo, ReactNode } from "react";
import { ErrorState } from "./ErrorState";

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
        // Tenta recuperar limpando caches ou apenas recarregando
        window.location.reload();
    };

    // Função auxiliar para traduzir erros técnicos do React
    getFriendlyMessage = (originalMessage: string) => {
        if (originalMessage.includes("Rendered fewer hooks")) {
            return "Erro interno de renderização (Conflito de Hooks).";
        }
        if (originalMessage.includes("is not a function")) {
            return "Erro de lógica: Função inválida chamada.";
        }
        if (originalMessage.includes("is not defined")) {
            return "Erro de variável não definida.";
        }
        if (originalMessage.includes("Failed to fetch")) {
            return "Falha de conexão com o servidor.";
        }
        return originalMessage; // Retorna original se não conhecer
    }

    render() {
        if (this.state.hasError) {
            const originalMsg = this.state.error?.message || "Erro desconhecido";
            const friendlyMsg = this.getFriendlyMessage(originalMsg);

            return (
                <div className="h-full flex flex-col items-center justify-center p-8 animate-in fade-in">
                    <ErrorState
                        type="generic"
                        message={friendlyMsg}
                        onRetry={this.handleRetry}
                    />
                </div>
            );
        }

        return this.props.children;
    }
}