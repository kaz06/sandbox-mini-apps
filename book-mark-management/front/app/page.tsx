'use client';

import { useState, useEffect} from "react";

export default function Home() {
  interface JsonData {
    name: string;
    data: string;
    tag: string[];
  }

  interface BookMarkNameTags {
    name: string;
    tags: string[];
  }
  
  const [jsonData, setJsonData] = useState<JsonData | null>(null);
  const [fileName, setFileName] = useState<string>("");
  const [tags, setTags] = useState<string>(""); 
  const [bookmarks, setBookmarks] = useState<BookMarkNameTags[]>([]);

  const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      setFileName(file.name); 
      const reader = new FileReader();
      reader.onload = (e) => {
        try {
          const parsedData = JSON.parse(e.target?.result as string);
          
          const formattedData = {
            name: file.name,       
            data: e.target?.result as string, 
            tag: tags.split(',').map(tag => tag.trim()), 
          };
          setJsonData(formattedData);
        } catch (error) {
          alert("Invalid JSON file");
        }
      };
      reader.readAsText(file);
    }
  };

  const handleTagChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setTags(event.target.value);
  };

  const handleSubmit = async () => {
    if (!jsonData) {
      alert("Please select a valid JSON file");
      return;
    }

    try {
      const response = await fetch("http://127.0.0.1:8080/bookmarkfile", {
        method: "POST",
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(jsonData),
      });

      if (response.ok) {
        alert("Data uploaded successfully!");
        fetchBookmarks(); 
        setTags(""); 
      } else {
        alert("Failed to upload data");
      }
    } catch (error) {
      console.error("Error uploading data:", error);
      alert("Error uploading data");
    }
  };

  const fetchBookmarks = async () => {
    try {
      const response = await fetch('http://127.0.0.1:8080/bookmarkfile');
      if (!response.ok) {
        throw new Error('Failed to fetch bookmarks');
      }
      const data: BookMarkNameTags[] = await response.json();
      setBookmarks(data);
    } catch (error) {
      console.error('Error fetching bookmarks:', error);
    }
  };

  useEffect(() => {
    fetchBookmarks(); 
  }, []);

  return (
    <main className="flex min-h-screen flex-col items-center justify-between p-24">
      <div className="mb-5 block w-full">
        <label className="block mb-2 text-sm font-medium text-gray-900 dark:text-white">
          BookMark JSON
        </label>
        <input
          type="file"
          id="file"
          accept=".json"
          onChange={handleFileChange}
          className="
            bg-gray-50 
            border border-gray-300 
            text-gray-900 text-sm 
            rounded-lg 
            focus:ring-blue-500 
            focus:border-blue-500 
            block w-full p-5
            dark:bg-gray-700 
            dark:border-gray-600 
            dark:placeholder-gray-400 
            dark:text-white 
            dark:focus:ring-blue-500 
            dark:focus:border-blue-500
          "
          required
        />
      </div>
      
      <div className="mb-5 block w-full">
        <label className="block mb-2 text-sm font-medium text-gray-900 dark:text-white">
          Tags (comma separated)
        </label>
        <input
          type="text"
          value={tags}
          onChange={handleTagChange}
          className="
            bg-gray-50 
            border border-gray-300 
            text-gray-900 text-sm 
            rounded-lg 
            focus:ring-blue-500 
            focus:border-blue-500 
            block w-full p-5
            dark:bg-gray-700 
            dark:border-gray-600 
            dark:placeholder-gray-400 
            dark:text-white 
            dark:focus:ring-blue-500 
            dark:focus:border-blue-500
          "
          placeholder="Enter tags separated by commas"
        />
      </div>

      <button
        onClick={handleSubmit}
        type="button"
        className="bg-blue-500 text-white p-3 rounded-lg"
      >
        Upload
      </button>
      
      <div className="w-full mt-10">
        {bookmarks.map((bookmark, index) => (
          <div key={index} className="mb-8 p-4 border border-gray-300 rounded-lg">
            <div className="mb-4">
              <h2 className="text-2xl font-bold">Name: {bookmark.name}</h2>
            </div>
            <div>
              <h3 className="text-xl font-semibold">Tags:</h3>
              <ul className="list-disc list-inside ml-4">
                {bookmark.tags.map((tag, tagIndex) => (
                  <li key={tagIndex} className="text-sm text-gray-600">{tag}</li>
                ))}
              </ul>
            </div>
          </div>
        ))}
      </div>
    </main>
  );
}
