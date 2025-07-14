import React from "react";

interface AvatarProps {
  name: string;
  size?: "sm" | "md" | "lg";
  isGroup?: boolean;
}

const Avatar: React.FC<AvatarProps> = ({
  name,
  size = "md",
  isGroup = false,
}) => {
  const sizeClasses = {
    sm: "w-8 h-8 text-sm",
    md: "w-10 h-10 text-base",
    lg: "w-14 h-14 text-lg",
  };

  return (
    <div
      className={`avatar placeholder ${isGroup ? "mask mask-squircle" : ""}`}
    >
      <div
        className={`${sizeClasses[size]} bg-green-500 text-white rounded-full flex items-center justify-center`}
      >
        {isGroup ? (
          <span className="font-bold">群</span>
        ) : (
          <span>{name.charAt(0)}</span>
        )}
      </div>
    </div>
  );
};

export default Avatar;
