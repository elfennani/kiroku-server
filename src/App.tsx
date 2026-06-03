import { Route, Routes } from "react-router";
import Home from "@/routes/Home.tsx";
import MediaRoute from "@/routes/Media.tsx";
import HomeLayout from "@/layout/HomeLayout.tsx";
import EpisodePlayerRoute from "@/routes/EpisodePlayer.tsx";

function App() {
  return (
    <Routes>
      <Route element={<HomeLayout />}>
        <Route path="/" element={<Home />} />
        <Route path="/media/:id" element={<MediaRoute />} />
        <Route path="/episode/:episodeId" element={<EpisodePlayerRoute />} />
      </Route>
    </Routes>
  );
}

export default App;
