<?php declare(strict_types = 1);

class HelloWorld
{
	public function sayHello(DateTimeImutable $date): void
	{
		$var = true;

		if($var == $var) {
			echo 'true';
		} else {
			echo 'false';
		}
	}
}
