{
  pkgs ? import <nixpkgs> { },
}:
pkgs.mkShell {
  buildInputs = with pkgs; [
    pkgs.pkg-config
    pkgs.xorg.libX11
    pkgs.xorg.libXrandr
    pkgs.xorg.libXinerama
    pkgs.xorg.libXcursor
    pkgs.xorg.libXi
    pkgs.shaderc 
    pkgs.vulkan-headers
    pkgs.vulkan-loader
    pkgs.vulkan-validation-layers 
    pkgs.mesa
    pkgs.extra-cmake-modules
    pkgs.wayland
    pkgs.wayland-protocols
    pkgs.libxkbcommon
  ];
  

  LD_LIBRARY_PATH= with pkgs; lib.makeLibraryPath [
    libGL
    libxkbcommon
  ]; 

  VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d"; 
}