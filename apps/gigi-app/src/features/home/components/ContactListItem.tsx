import React from 'react';
import Avatar from '../../../components/Avatar';
import type { Contact } from '../../../models/db';

interface ContactListItemProps {
  contact?: Contact;
  onClick: () => void;
}

const ContactListItem: React.FC<ContactListItemProps> = ({ contact, onClick }) => {
  if (!contact) {
    return null;
  }

  return (
    <div
      className="flex items-center p-3 border-b border-gray-200 hover:bg-gray-50 cursor-pointer"
      onClick={onClick}
    >
      <Avatar
        name={contact.name || 'Unknown'}
        size={40}
        address={contact.id}
      />
      <div className="ml-3">
        <div className="font-medium">{contact.name || 'Unknown'}</div>
      </div>
    </div>
  );
};

export default ContactListItem;
