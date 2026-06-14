import {StrictMode} from 'react'
import {createRoot} from 'react-dom/client'
import './index.css'
import App from './App.tsx'
import {BrowserRouter} from "react-router";
import {QueryClientProvider} from "@/api/query-client.tsx";

createRoot(document.getElementById('root')!).render(
    <StrictMode>
        <QueryClientProvider>
            <BrowserRouter>
                <App/>
            </BrowserRouter>
        </QueryClientProvider>
    </StrictMode>,
)
