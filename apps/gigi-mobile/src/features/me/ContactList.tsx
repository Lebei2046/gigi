import React, { useState, useMemo, useRef } from 'react'
import { HiOutlineSearch } from 'react-icons/hi'
import ContactListItem from './ContactListItem'
import { type Contact } from '@/models/db'
import { useAllContacts } from '@/models/contact'

// Utility function: Group and sort contacts
const groupAndSortContacts = (contacts: Contact[], searchTerm: string) => {
  const filtered = contacts.filter(contact =>
    contact.name.toLowerCase().includes(searchTerm.toLowerCase())
  )

  const grouped = filtered.reduce(
    (acc, contact) => {
      const firstLetter = contact.name.charAt(0).toUpperCase()
      if (!acc[firstLetter]) {
        acc[firstLetter] = []
      }
      acc[firstLetter].push(contact)
      return acc
    },
    {} as Record<string, Contact[]>
  )

  return Object.entries(grouped).sort((a, b) => a[0].localeCompare(b[0]))
}

const ContactList: React.FC = () => {
  const [searchTerm, setSearchTerm] = useState('')
  const contacts = useAllContacts()
  const groupRefs = useRef<Record<string, HTMLDivElement | null>>({})

  // Cache grouping and sorting results
  const filteredGroups = useMemo(() => {
    if (!contacts || contacts.length === 0) return []
    return groupAndSortContacts(contacts, searchTerm)
  }, [contacts, searchTerm])

  // Alphabet index
  const letters = useMemo(() => {
    return filteredGroups.map(([letter]) => letter)
  }, [filteredGroups])

  return (
    <div className="flex flex-col h-full bg-gray-50">
      {/* Search box */}
      <div className="sticky top-0 z-10 bg-white border-b border-gray-200 px-4 py-3">
        <div className="flex items-center bg-gray-100 rounded-xl px-4 py-3 focus-within:bg-white focus-within:ring-2 focus-within:ring-blue-500 focus-within:border-transparent transition-all">
          <HiOutlineSearch className="w-5 h-5 text-gray-400 mr-3" />
          <input
            type="text"
            placeholder="Search contacts..."
            className="flex-1 bg-transparent outline-none text-gray-900 placeholder-gray-500 text-sm"
            value={searchTerm}
            onChange={e => setSearchTerm(e.target.value)}
          />
        </div>
      </div>

      {/* Contact list */}
      <div className="flex-1 overflow-y-auto">
        {filteredGroups.length > 0 ? (
          <div className="px-4 py-2">
            {filteredGroups.map(([letter, group]) => (
              <div
                key={letter}
                ref={el => {
                  groupRefs.current[letter] = el
                }}
                className="mb-4"
              >
                <div className="sticky top-14 z-10 bg-blue-600 text-white px-3 py-2 rounded-lg mb-2 text-sm font-semibold shadow-sm">
                  {letter}
                </div>
                <div className="bg-white rounded-xl shadow-sm overflow-hidden">
                  {group.map(contact => (
                    <ContactListItem
                      key={contact.id}
                      name={contact.name}
                      peerId={contact.id}
                      onClick={function (): void {
                        console.log('TODO: Go to chat')
                      }}
                    />
                  ))}
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="flex items-center justify-center h-full">
            <div className="text-center py-12 px-6">
              <div className="w-20 h-20 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4">
                <svg
                  className="w-10 h-10 text-gray-400"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth="2"
                    d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"
                  ></path>
                </svg>
              </div>
              <h3 className="text-lg font-semibold text-gray-900 mb-2">
                No Contacts Found
              </h3>
              <p className="text-gray-600 text-sm">
                {searchTerm
                  ? 'No contacts match your search.'
                  : "You haven't added any contacts yet."}
              </p>
            </div>
          </div>
        )}
      </div>

      {/* Alphabet index */}
      {letters.length > 0 && (
        <div className="absolute right-3 top-24 bottom-20 flex flex-col justify-center bg-white/80 backdrop-blur rounded-full p-1 shadow-sm">
          {letters.map(letter => (
            <button
              key={letter}
              className="w-8 h-8 text-xs font-medium text-gray-600 hover:bg-blue-100 hover:text-blue-600 rounded-full transition-colors duration-200 flex items-center justify-center"
              onClick={() => {
                const element = groupRefs.current[letter]
                if (element) {
                  element.scrollIntoView({
                    behavior: 'smooth',
                    block: 'start',
                  })
                }
              }}
            >
              {letter}
            </button>
          ))}
        </div>
      )}
    </div>
  )
}

export default ContactList
