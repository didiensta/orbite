#! /bin/bash  

if [ ! -d ./$1/images ]; then
  mkdir ./$1/images
fi

for f in ./$1/positions/*.csv; do
	fileoutput=$(echo "./$1/$f" | cut -f 6 -d '/' | cut -f 1 -d '.')
	fileoutput="./$1/images/$fileoutput.png"
	gnuplot -e "filename='$f'; fileoutput='$fileoutput'" plotgnu
done
ffmpeg -y -i ./$1/images/%d.png ./$1/video.mp4
rm ./$1/images/*.png

