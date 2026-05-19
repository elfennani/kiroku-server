import {Route, Routes} from "react-router";
import Home from "@/routes/Home.tsx";
import MediaRoute from "@/routes/Media.tsx";

function App() {
    return (
        <Routes>
            <Route path="/" element={<Home/>}/>
            <Route path="/media/:id" element={<MediaRoute/>}/>
        </Routes>
    )
}

export default App
