import React from "react";
import Avatar from "./Avatar";

interface ContactListItemProps {
  id: string;
  name: string;
}

const ContactListItem: React.FC<ContactListItemProps> = ({ name }) => {
  return (
    <div className="flex items-center py-3 px-4 hover:bg-gray-50">
      <div className="flex-shrink-0 mr-3">
        <Avatar name={name} />
      </div>

      <div className="flex-1 min-w-0 border-b border-gray-100 pb-3">
        <h3 className="font-medium text-gray-900">{name}</h3>
      </div>
    </div>
  );
};

export default ContactListItem;
