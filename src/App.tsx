import { Route, Routes } from "react-router";
import Home from "@/routes/Home.tsx";
import MediaRoute from "@/routes/Media.tsx";
import HomeLayout from "@/layout/HomeLayout.tsx";

function App() {
  return (
    <Routes>
      <Route element={<HomeLayout />}>
        <Route path="/" element={<Home />} />
        <Route path="/media/:id" element={<MediaRoute />} />
      </Route>
    </Routes>
  );
}

export default App;
