import React, { useState, useMemo, useRef } from "react";
import { HiOutlineSearch } from "react-icons/hi";
import TopBar from "../components/TopBar";
import ContactListItem from "../components/ContactListItem";
import { type Contact } from "../../../models/db";
import { useAllContacts } from "../../../models/contact";

// Utility function: group and sort contacts
const groupAndSortContacts = (contacts: Contact[], searchTerm: string) => {
  const filtered = contacts.filter((contact) =>
    contact.name.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const grouped = filtered.reduce((acc, contact) => {
    const firstLetter = contact.name.charAt(0).toUpperCase();
    if (!acc[firstLetter]) {
      acc[firstLetter] = [];
    }
    acc[firstLetter].push(contact);
    return acc;
  }, {} as Record<string, Contact[]>);

  return Object.entries(grouped).sort((a, b) => a[0].localeCompare(b[0]));
};

const ContactList: React.FC = () => {
  const [searchTerm, setSearchTerm] = useState("");
  const contacts = useAllContacts();
  const groupRefs = useRef<Record<string, HTMLDivElement | null>>({});

  // Cache grouped and sorted results
  const filteredGroups = useMemo(() => {
    if (!contacts || contacts.length === 0) return [];
    return groupAndSortContacts(contacts, searchTerm);
  }, [contacts, searchTerm]);

  // Alphabetical index
  const letters = useMemo(() => {
    return filteredGroups.map(([letter]) => letter);
  }, [filteredGroups]);

  return (
    <div className="flex flex-col h-full">
      <TopBar title="Contacts" menuOpen={false} setMenuOpen={function (open: boolean): void {
        console.log(open);
      }} />
      {/* Search box */}
      <div className="sticky top-0 z-10 bg-gray-100 p-2">
        <div className="flex items-center bg-white rounded-lg px-3 py-2">
          <HiOutlineSearch className="w-5 h-5 text-gray-400 mr-2" />
          <input
            type="text"
            placeholder="Search"
            className="flex-1 outline-none text-sm"
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
        </div>
      </div>

      {/* Contact list */}
      <div className="flex-1 overflow-y-auto">
        {filteredGroups.length > 0 ? (
          filteredGroups.map(([letter, group]) => (
            <div
              key={letter}
              ref={(el) => (groupRefs.current[letter] = el)}
              className="py-2"
            >
              <div className="bg-gray-100 px-4 py-1 text-sm text-gray-500 sticky top-0">
                {letter}
              </div>
              <div className="bg-white">
                {group.map((contact) => (
                  <ContactListItem
                    key={contact.id}
                    contact={contact}
                    onClick={function (): void {
                      console.log("TODO: Go to chat");
                    }}
                  />
                ))}
              </div>
            </div>
          ))
        ) : (
          <div className="py-10 text-center text-gray-500">
            No matching contacts found
          </div>
        )}
      </div>

      {/* Alphabet index */}
      {letters.length > 0 && (
        <div className="absolute right-1 top-20 bottom-16 flex flex-col justify-center">
          {letters.map((letter) => (
            <button
              key={letter}
              className="text-xs px-1 py-0.5"
              onClick={() => {
                const element = groupRefs.current[letter];
                if (element) {
                  element.scrollIntoView({
                    behavior: "smooth",
                    block: "start",
                  });
                }
              }}
            >
              {letter}
            </button>
          ))}
        </div>
      )}
    </div>
  );
};

export default ContactList;
