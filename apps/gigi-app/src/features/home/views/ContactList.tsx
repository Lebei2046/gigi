import React, { useState } from "react";
import { HiOutlineSearch } from "react-icons/hi";
import ContactListItem from "../components/ContactListItem";
import { contacts } from "../../../data/users";

const ContactList: React.FC = () => {
  const [searchTerm, setSearchTerm] = useState("");

  // 按字母分组联系人 (排除搜索不匹配的)
  const groupedContacts = contacts
    .filter((contact) =>
      contact.name.toLowerCase().includes(searchTerm.toLowerCase())
    )
    .reduce((acc, contact) => {
      const firstLetter = contact.name.charAt(0).toUpperCase();
      if (!acc[firstLetter]) {
        acc[firstLetter] = [];
      }
      acc[firstLetter].push(contact);
      return acc;
    }, {} as Record<string, typeof contacts>);

  // 按字母顺序排序分组
  const filteredGroups = Object.entries(groupedContacts).sort((a, b) =>
    a[0].localeCompare(b[0])
  );

  // 字母索引
  const letters = filteredGroups.map(([letter]) => letter);

  return (
    <div className="flex flex-col h-full">
      {/* 搜索框 */}
      <div className="sticky top-0 z-10 bg-gray-100 p-2">
        <div className="flex items-center bg-white rounded-lg px-3 py-2">
          <HiOutlineSearch className="w-5 h-5 text-gray-400 mr-2" />
          <input
            type="text"
            placeholder="搜索"
            className="flex-1 outline-none text-sm"
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
        </div>
      </div>

      {/* 联系人列表 */}
      <div className="flex-1 overflow-y-auto">
        {filteredGroups.length > 0 ? (
          filteredGroups.map(([letter, group]) => (
            <div key={letter} id={`group-${letter}`} className="py-2">
              <div className="bg-gray-100 px-4 py-1 text-sm text-gray-500 sticky top-12">
                {letter}
              </div>
              <div className="bg-white">
                {group.map((contact) => (
                  <ContactListItem
                    key={contact.id}
                    id={contact.id}
                    name={contact.name}
                  />
                ))}
              </div>
            </div>
          ))
        ) : (
          <div className="py-10 text-center text-gray-500">
            没有找到匹配的联系人
          </div>
        )}
      </div>

      {/* 字母索引 */}
      {letters.length > 0 && (
        <div className="absolute right-1 top-20 bottom-16 flex flex-col justify-center">
          {letters.map((letter) => (
            <button
              key={letter}
              className="text-xs px-1 py-0.5"
              onClick={() => {
                const element = document.getElementById(`group-${letter}`);
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
