import React from "react";
import { FaCompass, FaCamera, FaStar, FaSmile } from "react-icons/fa";
import TopBar from "../components/TopBar";

const discoverItems = [
  {
    icon: <FaCompass className="text-xl text-green-500" />,
    label: "朋友圈",
  },
  {
    icon: <FaCamera className="text-xl text-blue-400" />,
    label: "扫一扫",
  },
  {
    icon: <FaStar className="text-xl text-yellow-400" />,
    label: "看一看",
  },
  {
    icon: <FaSmile className="text-xl text-pink-400" />,
    label: "搜一搜",
  },
];

const DiscoverPage: React.FC = () => (
  <div className="flex flex-col h-full">
    <TopBar title="发现" menuOpen={false} setMenuOpen={function (open: boolean): void {
      console.log(open);
    } } />
    <div className="space-y-2">
      {discoverItems.map((item, /*idx*/) => (
        <div
          key={item.label}
          className="flex items-center bg-white px-4 py-3 rounded-lg shadow-sm"
        >
          {item.icon}
          <span className="ml-4 text-base">{item.label}</span>
        </div>
      ))}
    </div>
  </div>
);

export default DiscoverPage;
