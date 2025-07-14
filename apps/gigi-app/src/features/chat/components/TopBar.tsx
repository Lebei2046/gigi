// src/components/TopBar.tsx
import { FiArrowLeft, FiMoreVertical } from 'react-icons/fi';

const TopBar = ({
  groupName,
  onBack,
}: {
  groupName: string;
  onBack: () => void;
}) => (
  <div className="flex items-center justify-between p-3 bg-gray-100">
    <button type="button" onClick={onBack} className="p-2">
      <FiArrowLeft size={24} />
    </button>
    <div className="font-semibold">{groupName}</div>
    <button type="button" className="p-2">
      <FiMoreVertical size={24} />
    </button>
  </div>
);

export default TopBar;
